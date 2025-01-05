use anyhow::Result;
use build_debugger_in_rust::{debugger, error::DebuggerError};
use rustyline::{error::ReadlineError, DefaultEditor};

fn main() -> Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Usage: {} <path>", std::env::args().next().unwrap());
        std::process::exit(1);
    };

    let mut debugger = debugger::Debugger::new(&path)?;
    debugger.start()?;

    let mut rl = DefaultEditor::new()?;

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                match line.trim() {
                    "quit" | "exit" => break,
                    "c" => match debugger.cont() {
                        Ok(_) => {}
                        Err(e) => {
                            if let Some(debug_err) = e.downcast_ref::<DebuggerError>() {
                                match debug_err {
                                    DebuggerError::Finished => {}
                                    DebuggerError::NoChildProcess => {
                                        eprintln!("No child process");
                                    }
                                    DebuggerError::SigIll => {
                                        eprintln!("SIGILL received");
                                    }
                                }
                            } else {
                                eprintln!("{}", e);
                            }
                            break;
                        }
                    },
                    _ => {
                        let mut parts = line.split(" ");
                        let Some(cmd) = parts.next() else {
                            continue;
                        };
                        let args = parts.collect::<Vec<_>>();
                        match cmd {
                            "b" => {
                                if args.len() != 1 {
                                    println!("Usage: b <address>");
                                    continue;
                                }
                                debugger.set_breakpoint(args[0])?
                            }
                            _ => println!("Unknown command: {}", cmd),
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // 履歴を保存
    rl.save_history("history.txt")?;

    Ok(())
}
