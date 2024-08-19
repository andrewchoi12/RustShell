use std::process::{Command, Stdio};
use std::io::{Write, stdin, stdout};
use std::fs::OpenOptions;

// Prompt user for command and read input
fn get_input() -> String {
    print!("rush$ ");
    stdout().flush().unwrap();
    let mut user_input = String::new();
    stdin().read_line(&mut user_input).unwrap();
    user_input.trim().to_string()
}

// Process and execute user's command
fn do_command(cmd_parts: &[&str], stdin_path: Option<&str>, stdout_path: Option<&str>, append: bool, run_in_background: bool) -> Result<(), String> {
    if let Some(executable) = cmd_parts.first() {
        let mut cmd = Command::new(executable);
        if cmd_parts.len() > 1 {
            cmd.args(&cmd_parts[1..]);
        }
        apply_redirection(&mut cmd, stdin_path, stdout_path, append)?;
        let mut child_process = cmd.spawn().map_err(|e| e.to_string())?;
        if !run_in_background {
            child_process.wait().map_err(|e| e.to_string())?;
        }
        Ok(())
    } // End outer if
    else {
        Err("Command not specified".to_string())
    }
}

// Set up i/o redirections for cmd
fn apply_redirection(cmd: &mut Command, stdin_file: Option<&str>, stdout_file: Option<&str>, append: bool) -> Result<(), String> {
    if let Some(input_path) = stdin_file {
        let file = OpenOptions::new().read(true).open(input_path).map_err(|e| e.to_string())?;
        cmd.stdin(file);
    }
    if let Some(output_path) = stdout_file {
        let file = OpenOptions::new()
            // Enable writing
            .write(true)
            // Create file
            .create(true)
            // Append
            .append(append)
            // clear file
            .truncate(!append)
            .open(output_path)
            .map_err(|e| e.to_string())?;
        cmd.stdout(file);
    } else {
        cmd.stdout(Stdio::inherit());
    }
    Ok(())

}

fn parse_command<'a>(tokens: &[&'a str]) -> Result<(Vec<&'a str>, Option<&'a str>, Option<&'a str>, bool), String> {
    // holds cmd + args
    let mut arguments = Vec::new();
    // holds path for input
    let mut input_file = None;
    // holds path for ouput
    let mut output_file = None;
    // tracks if need append
    let mut append = false;
    let mut out_redirect_count = 0;

    let mut index = 0;

    while index < tokens.len() {
        match tokens[index] {
            ">" => {
                if let Some(file_path) = tokens.get(index + 1) {
                    output_file = Some(*file_path);
                    append = false;
                    out_redirect_count += 1;
                    index += 1
                } else {
                    return Err("> must have a file afterwards!".to_string());
                }
            }
            // input redirections
            "<" => {
                if let Some(file_path) = tokens.get(index + 1) {
                    input_file = Some(*file_path);
                    index += 1;
                } else {
                    return Err("< must have a file afterwards!".to_string());
                }
            }  
            // Handle append output
            ">>" => {
                if let Some(file_path) = tokens.get(index + 1) {
                    output_file = Some(*file_path);
                    append = true;
                    out_redirect_count += 1;
                    index += 1;
                } else {
                    return Err(">> must have a file afterwards!".to_string());
                }
            }
            _ => arguments.push(tokens[index]),
        } // End of match
        index += 1;
    } // End of while

    if out_redirect_count > 1 {
        return Err("Can't have multiple output redirects".to_string());
    }

    Ok((arguments, input_file, output_file, append))
}

fn process_input(input : &str) -> Result<(), String> {
    let background = input.ends_with('&');
    let mut tokens = input.split_whitespace().collect::<Vec<&str>>();
    if background {
        tokens.pop();
    }

    let (cmd_parts, stdin_path, stdout_path, append) = parse_command(&tokens)?;

    do_command(&cmd_parts, stdin_path, stdout_path, append, background)
}


fn main() {
    loop {
        let user_input = get_input();
        if user_input.to_lowercase() == "exit" {
            break;
        }
        if let Err(e) = process_input(&user_input) {
            eprintln!("Error: {}", e);
            continue;
        }
    }
}

