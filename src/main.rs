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

fn init_container(command: &str, args: &[String], rootfs: &str) -> isize {
    set_hostname(HOSTNAME);
    mount_proc(rootfs).unwrap();
    mount_rootfs(rootfs).unwrap();

    let args = args
        .iter()
        .map(|arg| CString::new(arg.clone().into_bytes()).unwrap())
        .collect::<Vec<_>>();
    let env_vars = default_env_vars();
    unistd::execve(&CString::new(command).unwrap(), &args, &env_vars).unwrap();
    0
}

fn default_env_vars() -> Vec<CString> {
    let env_vars = vec![("PATH", "/usr/bin:/bin")];
    env_vars
        .into_iter()
        .map(|(key, value)| CString::new(format!("{}={}", key, value)).unwrap())
        .collect()
}

fn run_container(command: &str, args: &[String]) {
    let stack = &mut [0u8; STACK_SIZE];
    let child = Box::new(|| init_container(command, args, "rootfs"));
    let clone_flags = sched::CloneFlags::CLONE_NEWUTS
        | sched::CloneFlags::CLONE_NEWUSER
        | sched::CloneFlags::CLONE_NEWPID
        | sched::CloneFlags::CLONE_NEWNET
        | sched::CloneFlags::CLONE_NEWIPC
        | sched::CloneFlags::CLONE_NEWNS;
    let child_pid = sched::clone(child, stack, clone_flags, Some(Signal::SIGCHLD as i32))
        .expect("Failed to run container");
    wait::waitpid(child_pid, None).unwrap();
}

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let command = args[0].clone();
    args[0] = command.rsplit("/").next().unwrap().to_string();
    run_container(&command, &args);
}
