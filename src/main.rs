use std::process::Command;

use nix::{sched, unistd, NixResult};

const HOSTNAME: &'static str = "test";
const STACK_SIZE: usize = 1024;

fn set_hostname(hostname: &str) {
    unistd::sethostname(hostname.as_bytes()).expect("Cannot set hostname")
}

fn init_container(command: &str) -> isize {
    set_hostname(HOSTNAME);
    Command::new(command)
        .spawn()
        .expect("Failed to execute the command")
        .wait()
        .expect("Failed to finish the command")
        .code()
        .unwrap_or(-1)
}

fn run_container(command: &str) {
    let stack = &mut [0u8; STACK_SIZE];
    let child = Box::new(|| init_container(command));
    sched::clone(child, stack, clone_flags, None).expect("Failed to run container");
}

fn main() {
    let command = "/bin/sh";
    run_container(command);
}
