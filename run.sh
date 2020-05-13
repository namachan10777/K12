#!/bin/bash

GDBPORT=10000
SSHPORT=10022

echo "gdb port:"$GDBPORT
echo "ssh port:"$SSHPORT

qemu-system-x86_64 \
	--enable-kvm \
	-m 1024 \
	-boot order=d \
	-bios ./OVMF.fd \
	-gdb tcp::$GDBPORT \
	-net user,hostfwd=tcp::$SSHPORT-:22 -net nic \
	./K12
