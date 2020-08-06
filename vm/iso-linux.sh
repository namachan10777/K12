#!/bin/bash
SSHPORT=10022

qemu-system-x86_64 \
	--enable-kvm \
	-m 4096 \
	-boot order=d \
	-bios /usr/share/ovmf/x64/OVMF.fd \
	-cdrom $1 \
	-net user,hostfwd=tcp::$SSHPORT-:22 -net nic \
	./linux.qcow2
