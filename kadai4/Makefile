#
# Makefile for the kernel land lambda calcuration
#

# Lambda
obj-m := lambda.o 

all:
	make -C ../kernel M=$(shell pwd) modules

clean:
	make -C ../kernel M=$(shell pwd) clean
