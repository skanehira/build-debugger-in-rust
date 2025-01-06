use crate::{
    breakpoint::BreakPoint,
    error::DebuggerError,
    register::{self, Register},
    source_code_locator::SourceCodeLocator,
    source_code_printer::print_source_code,
};
use anyhow::{bail, Context as _, Result};
use nix::{
    sys::{
        ptrace::{self, AddressType},
        signal::Signal,
        wait::{waitpid, WaitStatus},
    },
    unistd::execve,
};
use std::{collections::HashMap, ffi::CString};

struct Process {
    pid: nix::unistd::Pid,
    status: WaitStatus,
}

pub struct Debugger<'a> {
    path: &'a str,
    process: Option<Process>,
    breakpoints: HashMap<u64, BreakPoint>,
    locator: SourceCodeLocator,
}

impl<'a> Debugger<'a> {
    pub fn new(path: &'a str) -> Result<Self> {
        Ok(Self {
            path,
            process: None,
            breakpoints: HashMap::new(),
            locator: SourceCodeLocator::new(path)?,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        unsafe {
            match nix::unistd::fork()? {
                nix::unistd::ForkResult::Parent { child } => {
                    self.process = Some(Process {
                        pid: child,
                        status: waitpid(child, None)?,
                    });
                    println!("Watching child pid: {}", child);
                    Ok(())
                }
                nix::unistd::ForkResult::Child => {
                    ptrace::traceme()?;
                    let path = CString::new(self.path)?;
                    execve::<CString, CString>(&path, &[], &[])?;
                    Ok(())
                }
            }
        }
    }

    fn wait(&mut self) -> Result<WaitStatus> {
        let p = self
            .process
            .as_mut()
            .with_context(|| DebuggerError::NoChildProcess)?;

        p.status = waitpid(p.pid, None)?;

        if matches!(p.status, WaitStatus::Exited(_, _)) {
            bail!(DebuggerError::Finished);
        }

        Ok(p.status)
    }

    pub fn cont(&mut self) -> Result<()> {
        self.step_over_breakpoint_if_needed()?;

        let p = self
            .process
            .as_ref()
            .with_context(|| DebuggerError::NoChildProcess)?;

        ptrace::cont(p.pid, None)?;

        let status = self.wait()?;

        match status {
            WaitStatus::Stopped(_, signal) => match signal {
                Signal::SIGTRAP => {
                    self.on_breakpoint_hit()?;
                }
                Signal::SIGILL => {
                    bail!(DebuggerError::SigIll);
                }
                _ => {
                    return self.cont();
                }
            },
            _ => {
                // ignore other signals
            }
        }

        Ok(())
    }

    pub fn set_breakpoint(&mut self, address: &str) -> Result<()> {
        let p = self
            .process
            .as_mut()
            .with_context(|| DebuggerError::NoChildProcess)?;

        let addr = u64::from_str_radix(address, 16)? as AddressType;

        let bp = BreakPoint::new(p.pid, addr)?;
        self.breakpoints.insert(addr as u64, bp);

        Ok(())
    }

    pub fn get_pc(&self) -> Result<u64> {
        let p = self
            .process
            .as_ref()
            .with_context(|| DebuggerError::NoChildProcess)?;
        register::read_register(p.pid, Register::Rip)
    }

    pub fn set_pc(&self, value: u64) -> Result<()> {
        let p = self
            .process
            .as_ref()
            .with_context(|| DebuggerError::NoChildProcess)?;
        register::write_register(p.pid, Register::Rip, value)
    }

    pub fn on_breakpoint_hit(&mut self) -> Result<()> {
        let mut pc = self.get_pc()?;

        pc -= 1;

        self.set_pc(pc)?;

        self.print_source_code()?;

        Ok(())
    }

    pub fn step_over_breakpoint_if_needed(&mut self) -> Result<()> {
        let pc = self.get_pc()?;

        let enabled = self
            .breakpoints
            .get(&pc)
            .map(|bp| bp.is_enabled)
            .unwrap_or(false);

        if !enabled {
            return Ok(());
        }

        self.breakpoints.get_mut(&pc).unwrap().disable()?;

        let p = self
            .process
            .as_mut()
            .with_context(|| DebuggerError::NoChildProcess)?;

        ptrace::step(p.pid, None)?;

        self.wait()?;

        self.breakpoints.get_mut(&pc).unwrap().enable()?;

        Ok(())
    }

    fn print_source_code(&self) -> Result<()> {
        let pc = self.get_pc()?;

        if let Some((file, line)) = self.locator.get_source_location(pc)? {
            let file = std::fs::File::open(file)?;
            print_source_code(file, line as usize)?;
        }
        Ok(())
    }
}

impl Drop for Debugger<'_> {
    fn drop(&mut self) {
        if let Some(ref p) = self.process {
            if !matches!(p.status, WaitStatus::Exited(_, _)) {
                nix::sys::ptrace::kill(p.pid).expect("Failed to kill child process");
            }
        }
    }
}
