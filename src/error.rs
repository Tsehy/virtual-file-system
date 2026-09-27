pub fn error(text: &str) {
    eprintln!("\x1b[31mError:\x1b[0m {text}");
}

pub fn work_dir_not_dir() {
    error("WorkDir is not a directory!");
}

pub fn already_exists(name: &str) {
    error(&format!("'{name}' already exists!"));
}

pub fn no_parent_dir() {
    error("No parent directory!");
}

pub fn parent_dropped() {
    error("Parent is dropped from memory!");
}

pub fn not_found(name: &str) {
    error(&format!("'{name}' is not found!"));
}
