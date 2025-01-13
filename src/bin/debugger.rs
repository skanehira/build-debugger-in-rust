use anyhow::Result;
use build_debugger_in_rust::debugger;
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
                    _ => {
                        if do_command(&mut debugger, &line).is_err() {
                            break;
                        };
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

fn do_command(debugger: &mut debugger::Debugger, line: &str) -> Result<()> {
    match line {
        "c" => debugger.cont(),
        _ => {
            let mut parts = line.split(" ");
            let Some(cmd) = parts.next() else {
                return Ok(());
            };
            let args = parts.collect::<Vec<_>>();
            match cmd {
                "b" => {
                    if args.len() != 1 {
                        println!("Usage: b <address>");
                        return Ok(());
                    }
                    debugger.set_breakpoint(args[0])?
                }
                _ => println!("Unknown command: {}", cmd),
            }
            Ok(())
        }
    }
}
