#!/bin/bash

GDBPORT=10000
SSHPORT=10023

echo "gdb port:"$GDBPORT
echo "ssh port:"$SSHPORT

qemu-system-x86_64 \
	--enable-kvm \
	-smp 8 \
	-m 14332 \
	-gdb tcp::$GDBPORT \
	-net user,hostfwd=tcp::$SSHPORT-:22 -net nic \
	./freebsd.qcow2
