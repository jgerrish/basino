AS=avr-as
CC=avr-gcc
AR=avr-ar
LD=avr-ld

# Specifying a memory map here is critical to get code working with Rust
CFLAGS=-mmcu=atmega328p -Wall -Werror -g -O0 -T memory.x
LDFLAGS=-g -mavr5 -T memory.x
# LDFLAGS=-g -mavr5

SRCS = stack.S basino.S
OBJS = $(SRCS:.S=-basino.o)

PREFIXES=stack basino

all: libbasino.a

clean:
	rm -f basino-*.o libbasino*.a libbasino*.so stack*.o

%-basino.o: %.S
	$(CC) $(CFLAGS) -o $@ -c $<

libbasino.a: $(OBJS)
	$(LD) $(LDFLAGS) -static stack-basino.o basino-basino.o -o libbasino.a
