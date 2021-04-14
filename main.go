package main

import (
	"fmt"
	"os"
	"os/exec"
	"path"
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
	cmd.SysProcAttr = &syscall.SysProcAttr{
		Cloneflags: syscall.CLONE_NEWUTS |
			syscall.CLONE_NEWPID |
			syscall.CLONE_NEWNS,
		// syscall.CLONE_NEWUSER,
	}
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Println("Error", err)
		os.Exit(1)
	}
}

func child() error {
	fmt.Printf("Running %v as %d\n", os.Args[2:], os.Getpid())

	if err := syscall.Sethostname([]byte("container")); err != nil {
		fmt.Printf("Cannot change hostname: %v\n", err)
	}

	pivotBase := "./root"
	rootfs := "rootfs"
	if err := syscall.Mount("proc", path.Join(pivotBase, rootfs, "proc"), "proc", uintptr(syscall.MS_NOEXEC|syscall.MS_NOSUID|syscall.MS_NODEV), ""); err != nil {
		fmt.Printf("Cannot mount proc: %v\n", err)
		return err
	}

	if err := pivotRoot(pivotBase, rootfs); err != nil {
		return err
	}

	cmd := exec.Command(os.Args[2], os.Args[3:]...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		fmt.Printf("Command execution failed: %v\n", err)
		return err
	}
	return nil
}

func pivotRoot(pivotBase string, rootfs string) error {
	if err := syscall.Chdir(pivotBase); err != nil {
		fmt.Printf("Cannot change dir to %s: %v\n", pivotBase, err)
		return err
	}

	// Cannot mount 'root/rootfs' from '../root'.
	if err := syscall.Mount("rootfs", rootfs, "", syscall.MS_BIND|syscall.MS_REC, ""); err != nil {
		fmt.Printf("Cannot mount ./root/rootfs: %v\n", err)
		return err
	}

	oldrootfs := "oldrootfs"
	if err := os.MkdirAll(path.Join(rootfs, oldrootfs), 0700); err != nil {
		fmt.Printf("Cannot mkdir ./root/rootfs/oldrootfs: %v\n", err)
		return err
	}

	if err := syscall.PivotRoot("rootfs", path.Join(rootfs, oldrootfs)); err != nil {
		fmt.Printf("PivotRoot failed: %v\n", err)
		return err
	}

	if err := os.Chdir("/"); err != nil {
		fmt.Printf("Cannot change dir to /: %v\n", err)
		return err
	}

	if err := syscall.Unmount(path.Join("/", oldrootfs), syscall.MNT_DETACH); err != nil {
		fmt.Printf("Cannot unmount /oldrootfs: %v\n", err)
		return err
	}

	if err := os.RemoveAll(path.Join("/", oldrootfs)); err != nil {
		fmt.Printf("Cannot remove /oldrootfs: %v\n", err)
		return err
	}
	return nil
}
