#!/bin/bash

qemu-img create -f qcow2 linux.qcow2 16G
qemu-img create -f qcow2 freebsd.qcow2 16G
