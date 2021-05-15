use std::{env, ffi::CString};

use nix::{
    mount, sched,
    sys::{signal::Signal, wait},
    unistd,
};

const HOSTNAME: &'static str = "test";
const STACK_SIZE: usize = 1024;

fn set_hostname(hostname: &str) {
    unistd::sethostname(hostname).expect("Cannot set hostname")
}

fn mount_proc() {
    const PROC: &str = "proc";
    mount::mount(
        Some(PROC),
        "/proc",
        Some(PROC),
        mount::MsFlags::empty(),
        None as Option<&str>,
    )
    .expect("Failed to mount /proc");
}

fn init_container(command: &str) -> isize {
    set_hostname(HOSTNAME);
    mount_proc();

    let env_vars = env::vars()
        .map(|v| CString::new(format!("{}={}", v.0, v.1)).unwrap())
        .collect::<Vec<_>>();
    unistd::execve(
        &CString::new(command).unwrap(),
        &[CString::new(command).unwrap()],
        &env_vars,
    )
    .unwrap();
    return 0;
}

fn run_container(command: &str) {
    let stack = &mut [0u8; STACK_SIZE];
    let child = Box::new(|| init_container(command));
    let clone_flags = sched::CloneFlags::CLONE_NEWUTS
        | sched::CloneFlags::CLONE_NEWUSER
        | sched::CloneFlags::CLONE_NEWPID
        | sched::CloneFlags::CLONE_NEWNS;
    let child_pid = sched::clone(child, stack, clone_flags, Some(Signal::SIGCHLD as i32))
        .expect("Failed to run container");
    wait::waitpid(child_pid, None).unwrap();
}

fn main() {
    let command = "/bin/sh";
    run_container(command);
}
