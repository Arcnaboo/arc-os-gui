use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::process::{Command, Stdio, Child};
use std::path::Path;
use std::env;
use std::fs::OpenOptions;

fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    
    println!("Arc OS Shell - AI Native v0.1.0");
    println!("Type 'exit' to quit.");

    loop {
        let readline = rl.readline("arc-os $ ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let input = line.trim();

                if input.is_empty() { continue; }

                if input == "exit" { break; }

                execute_pipeline(input);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

fn execute_pipeline(input: &str) {
    // 1. Handle pipes
    let commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect();
    let mut previous_command: Option<Child> = None;

    for (i, cmd_str) in commands.iter().enumerate() {
        let is_last = i == commands.len() - 1;
        
        // 2. Handle redirection within each command segment (usually the last one)
        let mut redirect_file = None;
        let mut append = false;
        
        let mut final_cmd_str = *cmd_str;
        if final_cmd_str.contains(">>") {
            let parts: Vec<&str> = final_cmd_str.split(">>").collect();
            final_cmd_str = parts[0].trim();
            redirect_file = Some(parts[1].trim());
            append = true;
        } else if final_cmd_str.contains(">") {
            let parts: Vec<&str> = final_cmd_str.split(">").collect();
            final_cmd_str = parts[0].trim();
            redirect_file = Some(parts[1].trim());
        }

        let mut parts = final_cmd_str.split_whitespace();
        let command = match parts.next() {
            Some(c) => c,
            None => continue,
        };
        let args: Vec<&str> = parts.collect();

        // 3. Handle Built-ins (only if not in a pipe, or first in pipe for cd)
        if commands.len() == 1 {
            match command {
                "cd" => {
                    let new_dir = args.get(0).map_or("/", |&s| s);
                    if let Err(e) = env::set_current_dir(Path::new(new_dir)) {
                        eprintln!("cd error: {}", e);
                    }
                    return;
                }
                "pwd" => {
                    if let Ok(path) = env::current_dir() {
                        println!("{}", path.display());
                    }
                    return;
                }
                _ => {}
            }
        }

        // 4. Setup Input/Output
        let stdin = previous_command
            .map(|child| Stdio::from(child.stdout.unwrap()))
            .unwrap_or(Stdio::inherit());

        let stdout = if let Some(filename) = redirect_file {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(append)
                .truncate(!append)
                .open(filename);
            
            match file {
                Ok(f) => Stdio::from(f),
                Err(e) => {
                    eprintln!("Error opening file: {}", e);
                    return;
                }
            }
        } else if !is_last {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        // 5. Spawn Process
        let child = Command::new(command)
            .args(&args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn();

        match child {
            Ok(c) => {
                previous_command = Some(c);
            }
            Err(e) => {
                eprintln!("command not found: {} ({})", command, e);
                previous_command = None;
            }
        }
    }

    // 6. Wait for the last command in the pipeline
    if let Some(mut last_command) = previous_command {
        let _ = last_command.wait();
    }
}
