#!/bin/bash

modprobe lambda
mknod /dev/lambda c 62 1
chmod 666 /dev/lambda
