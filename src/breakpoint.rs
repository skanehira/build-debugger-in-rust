use anyhow::Result;
use nix::{
    sys::ptrace::{self, AddressType},
    unistd::Pid,
};

pub struct BreakPoint {
    pid: Pid,
    addr: AddressType,
    original_instruction: [u8; 8],
    pub is_enabled: bool,
}

impl BreakPoint {
    pub fn new(pid: Pid, addr: AddressType) -> Result<Self> {
        let mut bp = Self {
            pid,
            addr,
            original_instruction: [0; 8],
            is_enabled: false,
        };

        bp.enable()?;
        Ok(bp)
    }

    pub fn enable(&mut self) -> Result<()> {
        let data = ptrace::read(self.pid, self.addr)?;

        self.original_instruction = data.to_le_bytes();

        // `data & !0xff`の部分
        //          data:     = XXXX XXXX XXXX XXXX  (元の命令)
        //          !0xff:    = 1111 1111 0000 0000  (!でビット反転)
        //                    = XXXX XXXX 0000 0000  (下位8ビットをクリア)
        // `| 0xcc`の部分
        //  data & !0xff:     = XXXX XXXX 0000 0000  (下位8ビットが0)
        //          0xcc:     = 0000 0000 1100 1100
        //                    = XXXX XXXX 1100 1100  (下位8ビットにint3命令をセット)
        let new_data = (data & !0xff) | 0xcc;

        ptrace::write(self.pid, self.addr, new_data)?;

        self.is_enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        let data = ptrace::read(self.pid, self.addr)? as u64;
        let original_data = u64::from_le_bytes(self.original_instruction);

        let new_data = (data & !0xff) | (original_data & 0xff);

        ptrace::write(self.pid, self.addr, new_data as i64)?;

        self.is_enabled = false;
        Ok(())
    }
}
