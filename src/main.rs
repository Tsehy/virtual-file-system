mod error;
mod node;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use crate::node::{Command, Node};

fn main() {
    let root = Rc::new(RefCell::new(Node::root()));

    let mut work_dir = Rc::clone(&root);
    loop {
        print!(
            "user\x1b[32m@\x1b[0mvfs:\x1b[1;31m{}\x1b[0;32m $\x1b[0m ",
            node::pwd_command(&work_dir)
        );
        if let Err(err) = std::io::stdout().flush() {
            error::error(&format!("{err}"));
            continue;
        }

        let mut command = String::new();
        if let Err(err) = std::io::stdin().read_line(&mut command) {
            error::error(&format!("{err}"));
            continue;
        } else {
            command = command.trim().into();
        }

        if command.is_empty() {
            continue;
        }

        match Command::parse(&command) {
            Err(err) => error::error(&err),
            Ok(cmd) => match cmd {
                Command::Exit => break,
                Command::Pwd => println!("{}", node::pwd_command(&work_dir)),
                Command::Ls(dir) => node::ls_command(&dir, &work_dir),
                Command::MkDir(dir) => node::mkdir_command(&dir, &work_dir),
                Command::Cd(dir) => node::cd_command(&dir, &mut work_dir),
                Command::Touch(file) => node::touch_command(&file, &work_dir),
                Command::Cat(file) => node::cat_command(&file, &work_dir),
                Command::Write(file, content) => node::write_command(&file, &content, &work_dir),
                Command::Rm(name) => node::rm_command(&name, &work_dir),
                Command::Mv(name, new_name) => node::mv_command(&name, &new_name, &work_dir),
                Command::Tree(dir) => node::tree_command(&dir, &work_dir),
            },
        }
    }
}
