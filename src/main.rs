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
    exit_code: i32,
    cur_dir: PathBuf,
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
    if !dir.starts_with("/") && !dir.starts_with("./") && !dir.starts_with("..") {
        path.push("./");
    }
    path.push(dir);
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
    // let mut cmd = Command::new(program_name)
    //     .args(&commands[1..])
    //     .stdin(Stdio::inherit())
    //     .stdout(Stdio::inherit())
    //     .current_dir(&mut env.cur_dir)
    //     .spawn()?;
    // let status = cmd.wait().unwrap();
    // env.exit_with(status.code().unwrap());
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
        if buf.len() == 0 {
            continue;
        }
        let commands = buf.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
        match exec(&mut cur_env, commands) {
            Ok(_) => {},
            Err(msg) => println!("{}", msg),
        }
    }
}
