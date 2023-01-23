# basino
A BASIC compiler and set of tools for Atmel AVR devices

## Introduction

This project provides a small BASIC compiler and set of tools for
Atmel AVR devices.

Currently, it just includes some basic data structures implemented in
AVR assembly, and drivers in Nim and Rust to test the code base.

Hopefully, it can serve as an example of using Rust or Nim to quickly
iterate on ideas with AVR devices.  It's not meant to provide a
professional testbed for development.

Ratel and the avr-hal crate provide a starting point that could be
expanded to provide more automated testing.  This project doesn't
entirely follow best practices, it's mostly meant as a collection of
sample code.


## Requirements

### Toolchain

These are the packages from Debian-based systems needed to build the
project:

arduino-core-avr
avr-libc
avrdude
avrdude-doc
binutils-avr
gcc-avr
gdb-avr
make

On Ubuntu, gdb-avr is required, gdb-multiarch doesn't support AVR out-of-the-box.

Rust and Nim are required to build the Rust and Nim drivers.

#### Rust requirements

A nightly version of Rust is required for the avr crates.

#### Nim requirements

* Nim version 1.6.10.
* Ratel version 0.2.1

As of 2023-01-22 Nim 2.0 doesn't work with Ratel.


### Simulators and Emulators

These are useful for testing and running the code.

* simavr - a lean and mean Atmel AVR simulator for linux
  https://github.com/buserror/simavr.git
* qemu-system-avr


## Running

Build the projects:

$ make

This also creates symlinks to the libraries and sets up some other
things.  Linker flags were a pain to get working with each separate
language.  Users are welcome to contribute helpful improvements.

Then you can either use it in your own project, or try testing it with
Rust or Nim:

$ cd rust-basino
$ cargo build
$ cargo run

Build the Nim version:

$ cd basino_atmega328p
$ ratel build

Run with sim-avr
$ run_avr --mcu atmega328p basino_atmega328p
or with QEMU:
$ qemu-system-avr -M uno -nographic -serial mon:stdio -bios basino_atmega328p

## Debugging

Steps to get running and debugging:

In one terminal:

qemu-system-avr -s -S -M uno -nographic -serial mon:stdio -bios basino_atmega328p

In another terminal:

./basino/nim-avr-gdb ./basino_atmega328p/basino_atmega328p
