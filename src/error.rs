use std::error::Error;

#[derive(Debug)]
pub enum DebuggerError {
    Finished,
    NoChildProcess,
    SigIll,
}

impl std::fmt::Display for DebuggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DebuggerError::Finished => write!(f, "Debuggee process is finished"),
            DebuggerError::NoChildProcess => write!(f, "No child process"),
            DebuggerError::SigIll => write!(f, "SIGILL received"),
        }
    }
}

impl Error for DebuggerError {}
