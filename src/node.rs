use std::cell::RefCell;
use std::rc::{Rc, Weak};

type RcNode = Rc<RefCell<Node>>;

pub enum Node {
    File {
        name: String,
        content: String,
    },
    Directory {
        name: String,
        children: Vec<RcNode>,
        parent: Option<Weak<RefCell<Node>>>,
    },
}

impl Node {
    pub fn root() -> Self {
        Node::Directory {
            name: "root".into(),
            children: Vec::new(),
            parent: None,
        }
    }

    fn name(&self) -> &str {
        match self {
            Node::File { name, .. } => name,
            Node::Directory { name, .. } => name,
        }
    }

    fn node_type(&self) -> char {
        match self {
            Node::File { .. } => 'F',
            Node::Directory { .. } => 'D',
        }
    }

    fn parent(&self) -> &Option<Weak<RefCell<Node>>> {
        match self {
            Node::File { .. } => &None,
            Node::Directory { parent, .. } => parent,
        }
    }
}

pub enum Command {
    Exit,
    Pwd,
    Ls(Option<String>),
    Cd(String),
    MkDir(String),
    Touch(String),
    Rm(String),
    Cat(String),
    Write(String, String),
    Tree(Option<String>),
    Mv(String, String),
}

impl Command {
    pub fn parse(input: &str) -> Result<Command, String> {
        let parts = smart_split(input);
        match parts[0].as_str() {
            "exit" => Ok(Command::Exit),
            "pwd" => Ok(Command::Pwd),
            "ls" => Ok(Command::Ls(parts.get(1).map(|s| s.into()))),
            "cd" => {
                if parts.len() == 1 {
                    Err("Missing target directory!".into())
                } else {
                    Ok(Command::Cd(parts[1].clone()))
                }
            }
            "mkdir" => {
                if parts.len() == 1 {
                    Err("Missing directory name!".into())
                } else {
                    Ok(Command::MkDir(parts[1].clone()))
                }
            }
            "touch" => {
                if parts.len() == 1 {
                    Err("Missing file name!".into())
                } else {
                    Ok(Command::Touch(parts[1].clone()))
                }
            }
            "rm" => {
                if parts.len() == 1 {
                    Err("Missing argument!".into())
                } else {
                    Ok(Command::Rm(parts[1].clone()))
                }
            }
            "cat" => {
                if parts.len() == 1 {
                    Err("Missing file name!".into())
                } else {
                    Ok(Command::Cat(parts[1].clone()))
                }
            }
            "write" => {
                if parts.len() < 3 {
                    Err("Missing arguments!".into())
                } else {
                    Ok(Command::Write(parts[1].clone(), parts[2].clone()))
                }
            }
            "mv" => {
                if parts.len() < 3 {
                    Err("Missing arguments!".into())
                } else {
                    Ok(Command::Mv(parts[1].clone(), parts[2].clone()))
                }
            }
            "tree" => Ok(Command::Tree(parts.get(1).map(|t| t.into()))),
            _ => Err("Unknown command!".into()),
        }
    }
}

fn smart_split(text: &str) -> Vec<String> {
    let mut parts = Vec::new();

    let mut word_buffer = String::new();
    let mut quoted = false;
    for c in text.chars() {
        match c {
            '"' => quoted = !quoted,

            c if c.is_whitespace() && !quoted => {
                if !word_buffer.is_empty() {
                    parts.push(std::mem::take(&mut word_buffer))
                }
            }

            _ => word_buffer.push(c),
        }
    }

    if !word_buffer.is_empty() {
        parts.push(word_buffer);
    }

    parts
}

pub fn mkdir_command(dir_name: &str, work_dir: &RcNode) {
    match &mut *work_dir.borrow_mut() {
        Node::File { name, .. } => eprintln!("{name} is not a directory!"),
        Node::Directory { children, .. } => {
            if children.iter().any(|c| c.borrow().name() == dir_name) {
                eprintln!("{dir_name} already exists!");
            } else {
                children.push(Rc::new(RefCell::new(Node::Directory {
                    name: dir_name.into(),
                    children: Vec::new(),
                    parent: Some(Rc::downgrade(work_dir)),
                })));
            }
        }
    }
}

pub fn pwd_command(work_dir: &RcNode) -> String {
    match &*work_dir.borrow() {
        Node::File { .. } => "".into(),
        Node::Directory { name, parent, .. } => {
            let mut path: Vec<String> = Vec::new();
            path_rec(parent, &mut path);
            path.reverse();
            path.push(name.clone());
            path.join("/")
        }
    }
}

fn path_rec(node: &Option<Weak<RefCell<Node>>>, folders: &mut Vec<String>) {
    match node {
        None => (),
        Some(node) => match node.upgrade() {
            None => (),
            Some(node) => {
                folders.push(node.borrow().name().into());
                path_rec(node.borrow().parent(), folders);
            }
        },
    }
}

pub fn ls_command(dir: &Option<String>, work_dir: &RcNode) {
    match dir {
        None => match &*work_dir.borrow() {
            Node::File { .. } => eprintln!("\x1b[31mError:\x1b[0m Not a directory!"),
            Node::Directory { children, .. } => print_children(children),
        },
        Some(dir) => match resolve_dir_param(dir, work_dir) {
            Err(err) => eprintln!("{err}"),
            Ok(dir) => match &*dir.borrow() {
                Node::File { .. } => eprintln!("\x1b[31mError:\x1b[0m Not a directory!"),
                Node::Directory { children, .. } => print_children(children),
            },
        },
    }
}

fn print_children(children: &[RcNode]) {
    for child in children {
        let n = child.borrow();
        println!("\x1b[1;36m{}\x1b[0m {}", n.node_type(), n.name());
    }
}

pub fn cd_command(dir: &str, work_dir: &mut RcNode) {
    match resolve_dir_param(dir, work_dir) {
        Err(err) => eprintln!("{err}"),
        Ok(target_dir) => *work_dir = target_dir,
    }
}

fn resolve_dir_param(dir: &str, work_dir: &RcNode) -> Result<RcNode, &'static str> {
    if dir == ".." {
        match work_dir.borrow().parent() {
            None => Err("\x1b[31mError:\x1b[0m No parent directory!"),
            Some(parent) => parent
                .upgrade()
                .map(|p| Rc::clone(&p))
                .ok_or("\x1b[31mError:\x1b[0m Parent node is dropped!"),
        }
    } else {
        match &*work_dir.borrow() {
            Node::File { .. } => Err("\x1b[31mError:\x1b[0m Not a directory!"),
            Node::Directory { children, .. } => children
                .iter()
                .find(|c| {
                    let child = c.borrow();
                    child.name() == dir && child.node_type() == 'D'
                })
                .map(|c| Rc::clone(&c))
                .ok_or("\x1b[31mError:\x1b[0m Directory not found!"),
        }
    }
}

pub fn touch_command(file: &str, work_dir: &RcNode) {
    match &mut *work_dir.borrow_mut() {
        Node::File { name, .. } => eprintln!("\x1b[31mError:\x1b[0m {name} is not a directory!"),
        Node::Directory { children, .. } => {
            if children.iter().any(|c| c.borrow().name() == file) {
                eprintln!("\x1b[31mError:\x1b[0m {file} already exists!");
            } else {
                children.push(Rc::new(RefCell::new(Node::File {
                    name: file.into(),
                    content: "".into(),
                })));
            }
        }
    }
}

pub fn cat_command(file: &str, work_dir: &RcNode) {
    let target_file = match &*work_dir.borrow() {
        Node::File { .. } => None,
        Node::Directory { children, .. } => children
            .iter()
            .find(|c| match &*c.borrow() {
                Node::Directory { .. } => false,
                Node::File { name, .. } => name == file,
            })
            .map(Rc::clone),
    };
    match target_file {
        None => eprintln!("\x1b[31mError:\x1b[0m File '{file}' not found!"),
        Some(file) => match &*file.borrow() {
            Node::Directory { name, .. } => {
                eprintln!("\x1b[31mError:\x1b[0m {name} is not a file!")
            }
            Node::File { content, .. } => println!("{content}"),
        },
    }
}

pub fn write_command(file: &str, new_content: &str, work_dir: &RcNode) {
    match &*work_dir.borrow() {
        Node::File { name, .. } => eprintln!("\x1b[31mError:\x1b[0m {name} is not a directory!"),
        Node::Directory { children, .. } => match children.iter().find(|n| {
            let node = n.borrow();
            node.name() == file && node.node_type() == 'F'
        }) {
            None => eprintln!("\x1b[31mError:\x1b[0m Cannot find file '{file}'!"),
            Some(file) => match &mut *file.borrow_mut() {
                Node::Directory { name, .. } => {
                    eprintln!("\x1b[31mError:\x1b[0m {name} is not a file!")
                }
                Node::File { content, .. } => *content = new_content.into(),
            },
        },
    }
}

pub fn rm_command(name: &str, work_dir: &RcNode) {
    match &mut *work_dir.borrow_mut() {
        Node::File { name, .. } => eprintln!("\x1b[31mError:\x1b[0m {name} is not a directory!"),
        Node::Directory { children, .. } => {
            match children.iter().position(|n| n.borrow().name() == name) {
                None => eprintln!("\x1b[31mError:\x1b[0m Cannot find {name}!"),
                Some(index) => _ = children.swap_remove(index),
            }
        }
    }
}
