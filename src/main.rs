extern crate ansi_term;
extern crate dirs;

use std::io;
use std::io::{BufReader, BufRead, Write};
use std::process::{Command, Stdio};
use std::process;
use std::path::PathBuf;
use std::fs;

use ansi_term::Color::Red;
use dirs::*;

struct Env {
    exit_code: i32, // Exit code of previous command.
    cur_dir: PathBuf, // Current Directory.
}

impl Env {
    fn new(initial_dir: PathBuf) -> Env {
        Env {
            exit_code: 0,
            cur_dir: initial_dir,
        }
    }

    fn exit_with(&mut self, code: i32) {
        self.exit_code = code;
    }
}

fn builtin_cd(env: &mut Env, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::new();

    // When directory name isn't already Absolute path or Relative path,
    // make directory name as a Relative path.
    if !dir.starts_with("/") && !dir.starts_with("./") && !dir.starts_with("..") {
        path.push("./");
    }
    path.push(dir);

    // Convert path to Absolute path.
    // https://doc.rust-lang.org/std/fs/fn.canonicalize.html
    let absolute_path = fs::canonicalize(path)?;

    std::env::set_current_dir(absolute_path)?;
    env.cur_dir = std::env::current_dir().unwrap();
    Ok(())
}

fn exec_command(env: &mut Env, program_name: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(program_name)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .current_dir(&mut env.cur_dir)
        .spawn()?;
    
    let status = cmd.wait().unwrap();
    env.exit_with(status.code().unwrap());
    Ok(())
}

fn exec(env: &mut Env, commands: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let program_name = commands.get(0).unwrap();
    match program_name as &str {
        "cd" => builtin_cd(env, commands.get(1).unwrap())?,
        _ => exec_command(env, program_name, &commands[1..])?,
    }

    Ok(())
}

fn main() {
    let mut reader = BufReader::new(io::stdin());
    let home = home_dir().unwrap();
    let mut cur_env = Env::new(home);

    loop {
        let mut buf = String::new();

        if cur_env.exit_code != 0 {
            print!("{} ", Red.paint(format!("{}", cur_env.exit_code)));
        }
        std::env::set_current_dir(&cur_env.cur_dir).expect("Error occurred");
        print!("{}> ", cur_env.cur_dir.display());
        io::stdout().flush().expect("Error occurred");

        reader.read_line(&mut buf).expect("Error occurred");
        if buf.starts_with("exit") {
            process::exit(0);
        }

        let commands = buf.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
        if commands.len() == 0 {
            continue;
        }

        match exec(&mut cur_env, commands) {
            Ok(_) => {},
            Err(msg) => println!("{}", msg),
        }
    }
}
