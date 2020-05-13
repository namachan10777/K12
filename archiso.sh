#!/bin/bash

qemu-system-x86_64 --enable-kvm -m 1024 -boot order=d -bios ./OVMF.fd -cdrom $1 ./K12
