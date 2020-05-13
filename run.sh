#!/bin/bash

qemu-system-x86_64 --enable-kvm -m 1024 -boot order=d -bios ./OVMF.fd ./K12 -net user,hostfwd=tcp::10022-:22 -net nic
