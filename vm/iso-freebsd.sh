#!/bin/bash
SSHPORT=10022

qemu-system-x86_64 \
	-m 1024 \
	-cdrom $1 \
	--enable-kvm \
	-net user,hostfwd=tcp::$SSHPORT-:22 -net nic \
	-hda ./freebsd.qcow2
