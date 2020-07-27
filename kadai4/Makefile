#
# Makefile for the kernel land lambda calcuration
#

# Lambda
obj-m := lambda.o 
all:
	make -C /lib/modules/$(shell uname -r)/build M=$(shell pwd) modules

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(shell pwd) clean
