use std::{env, ffi::CString, fs, path::PathBuf};

use nix::{
    mount, sched,
    sys::{signal::Signal, stat, wait},
    unistd,
};

const HOSTNAME: &str = "test";
const STACK_SIZE: usize = 1024;

fn set_hostname(hostname: &str) {
    unistd::sethostname(hostname).expect("Cannot set hostname")
}

fn mount_proc(rootfs: &str) -> nix::Result<()> {
    const PROC: &str = "proc";
    let mount_to = PathBuf::from(rootfs).join(PROC);
    mount::mount(
        Some(PROC),
        &mount_to,
        Some(PROC),
        mount::MsFlags::empty(),
        None::<&str>,
    )
}

fn mount_rootfs(rootfs: &str) -> nix::Result<()> {
    const OLD_ROOT: &str = "oldroot";
    let oldrootfs = PathBuf::from(rootfs).join(OLD_ROOT);
    mount::mount(
        Some(rootfs),
        rootfs,
        None::<&str>,
        mount::MsFlags::MS_BIND | mount::MsFlags::MS_REC,
        None::<&str>,
    )?;
    if let Err(err) = unistd::mkdir(
        &oldrootfs,
        stat::Mode::S_IRWXU | stat::Mode::S_IRWXG | stat::Mode::S_IRWXO,
    ) {
        if err != nix::Error::Sys(nix::errno::Errno::EEXIST) {
            return Err(err);
        }
    }
    unistd::pivot_root(rootfs, &oldrootfs)?;
    unistd::chdir("/").unwrap();
    mount::umount2("/oldroot", mount::MntFlags::MNT_DETACH)?;
    fs::remove_dir_all(OLD_ROOT).unwrap();
    Ok(())
}

fn init_container(command: &str, rootfs: &str) -> isize {
    set_hostname(HOSTNAME);
    mount_proc(rootfs).unwrap();
    mount_rootfs(rootfs).unwrap();

    let env_vars = env::vars()
        .map(|v| CString::new(format!("{}={}", v.0, v.1)).unwrap())
        .collect::<Vec<_>>();
    unistd::execve(
        &CString::new(command).unwrap(),
        &[CString::new(command).unwrap()],
        &env_vars,
    )
    .unwrap();
    0
}

fn run_container(command: &str) {
    let stack = &mut [0u8; STACK_SIZE];
    let child = Box::new(|| init_container(command, "rootfs"));
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
