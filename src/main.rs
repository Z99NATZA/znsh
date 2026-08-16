use znsh::parser;

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

fn main() -> io::Result<()> {
    const SHELL_NAME: &str = "znsh";

    let mut first_loop = true;

    loop {
        print!("{}{SHELL_NAME} $ ", if first_loop { "" } else { "\n" });
        first_loop = false;

        io::stdout().flush()?;

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;

        if bytes_read == 0 {
            println!();
            break;
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let tokens = parser::tokenize(input);

        let Some((cmd, args)) = tokens.split_first() else {
            continue;
        };

        let cmd = cmd.as_str();

        match cmd {
            "exit" | "q" | "quit" => {
                break;
            }
            "cd" => {
                if args.len() > 1 {
                    eprintln!(r#"{SHELL_NAME}: "cd": too many arguments"#);
                    continue;
                }

                let target = args
                    .first()
                    .map(|p| PathBuf::from(p.as_str()))
                    .or_else(|| env::var_os("HOME").map(PathBuf::from));

                match target {
                    Some(path) => {
                        if let Err(error) = env::set_current_dir(&path) {
                            eprintln!(r#"{SHELL_NAME}: "{}": {error}"#, path.display());
                        }
                    }
                    None => {
                        eprintln!(r#"{SHELL_NAME}: "cd": HOME is not set"#);
                    }
                }
            }
            "echo" => {
                println!("{}", args.join(" "));
            }
            _ => {
                let cmd = match cmd {
                    "cls" => "clear",
                    _ => cmd,
                };

                match Command::new(cmd).args(args).status() {
                    Ok(status) => {
                        if !status.success() {
                            eprintln!(r#"{SHELL_NAME}: "{cmd}": command exited with {status}"#);
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        eprintln!(r#"{SHELL_NAME}: "{cmd}": command not found"#);
                    }
                    Err(error) => {
                        eprintln!(r#"{SHELL_NAME}: "{cmd}": failed to run: {error}"#);
                    }
                }
            }
        }
    }

    Ok(())
}
