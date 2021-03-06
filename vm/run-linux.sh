#!/bin/bash

GDBPORT=10000
SSHPORT=10022

echo "gdb port:"$GDBPORT
echo "ssh port:"$SSHPORT

qemu-system-x86_64 \
	--enable-kvm \
	-m 4096 \
	-boot order=d \
	-bios /usr/share/ovmf/x64/OVMF.fd \
	-gdb tcp::$GDBPORT \
	-net user,hostfwd=tcp::$SSHPORT-:22 -net nic \
	./linux.qcow2
