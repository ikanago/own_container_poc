package main

import (
	"fmt"
	// "io/ioutil"
	"os"
	"os/exec"
	"syscall"
)

func main() {
	switch os.Args[1] {
	case "run":
		parent()
	case "child":
		child()
	default:
		panic("Unexpected command")
	}
}

func parent() {
	fmt.Printf("Running %v as %d\n", os.Args[2:], os.Getpid())

	cmd := exec.Command("/proc/self/exe", append([]string{"child"}, os.Args[2:]...)...)
	cmd.SysProcAttr = &syscall.SysProcAttr {
		Cloneflags: syscall.CLONE_NEWUTS | syscall.CLONE_NEWPID | syscall.CLONE_NEWNS,
	}
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Println("Error", err)
		os.Exit(1)
	}
}

func child() {
	fmt.Printf("Running %v as %d\n", os.Args[2:], os.Getpid())

	try(syscall.Sethostname([]byte("container")))
	try(syscall.Mount("proc", "/root/rootfs/proc", "proc", uintptr(syscall.MS_NOEXEC | syscall.MS_NOSUID | syscall.MS_NODEV), ""))
	try(syscall.Chdir("/root"))
	try(syscall.Mount("rootfs", "/root/rootfs", "", syscall.MS_BIND | syscall.MS_REC, ""))
	try(os.MkdirAll("/root/rootfs/oldrootfs", 0700))
	try(syscall.PivotRoot("rootfs", "/root/rootfs/oldrootfs"))
	fmt.Println("reached")
	try(syscall.Unmount("/oldrootfs", syscall.MNT_DETACH))
	try(os.RemoveAll("/oldrootfs"))
	try(os.Chdir("/"))

	cmd := exec.Command(os.Args[2], os.Args[3:]...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	try(cmd.Run())

}

func try(err error) {
	if err != nil {
		panic(err)
	}
}
