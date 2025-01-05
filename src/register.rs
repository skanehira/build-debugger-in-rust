use nix::{sys::ptrace, unistd::Pid};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum Register {
    Rbp, // ベースポインタ
    Rip, // インストラクションポインタ（プログラムカウンタ）
    Rsp, // スタックポインタ
}

pub fn read_register(pid: Pid, register: Register) -> Result<u64> {
    let regs = ptrace::getregs(pid)?;

    let reg = match register {
        Register::Rbp => regs.rbp,
        Register::Rip => regs.rip,
        Register::Rsp => regs.rsp,
    };

    Ok(reg as u64)
}

pub fn write_register(pid: Pid, register: Register, value: u64) -> Result<()> {
    let mut regs = ptrace::getregs(pid)?;

    match register {
        Register::Rbp => regs.rbp = value as u64,
        Register::Rip => regs.rip = value as u64,
        Register::Rsp => regs.rsp = value as u64,
    };

    Ok(ptrace::setregs(pid, regs)?)
}
