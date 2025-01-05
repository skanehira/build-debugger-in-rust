use anyhow::Result;
use nix::libc;
use nix::sys::ptrace;
use std::ffi::CString;

/// # Safety
pub unsafe fn execute_debuggee_program(path: &str) -> Result<()> {
    match nix::unistd::fork()? {
        nix::unistd::ForkResult::Parent { child } => {
            println!("Parent process watching child pid: {}", child);
            let mut status = 0;
            let cpid = child.as_raw();
            libc::waitpid(cpid, &mut status, 0);

            println!("Continue child process");
            ptrace::cont(child, None)?;

            libc::waitpid(cpid, &mut status, 0);
            println!("Child process has exited");
        }
        nix::unistd::ForkResult::Child => {
            ptrace::traceme()?;
            let path = CString::new(path)?;
            nix::unistd::execve::<CString, CString>(&path, &[], &[])?;
        }
    }
    Ok(())
}
