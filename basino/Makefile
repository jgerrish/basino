AS=avr-as
CC=avr-gcc
AR=avr-ar
LD=avr-ld

# Specifying a memory map here is critical to get code working with Rust
CFLAGS=-mmcu=atmega328p -Wall -Werror -g -O0 -T memory.x -r
LDFLAGS=-g -mavr5 -T memory.x -r
ARFLAGS=rcs --target elf32-avr
# LDFLAGS=-g -mavr5

SRCS = stack.S queue.S il.S inst.S basino.S
OBJS = $(SRCS:.S=-basino.o)

PREFIXES=stack queue il inst basino

all: libbasino.a

clean:
	rm -f basino-*.o libbasino*.a libbasino*.so stack*.o queue*.o il*.o inst*.o

%-basino.o: %.S
	$(CC) $(CFLAGS) -o $@ -c $<

libbasino.a: $(OBJS)
	$(LD) $(LDFLAGS) -static stack-basino.o queue-basino.o il-basino.o inst-basino.o basino-basino.o -o libbasino.a
#	$(AR) $(ARFLAGS) stack-basino.o queue-basino.o il-basino.o basino-basino.o libbasino.a
