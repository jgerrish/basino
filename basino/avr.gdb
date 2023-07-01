# emacs invocation: avr-gdb -i=mi -x /home/josh/src/asm/basino/basino/avr.gdb -d /home/josh/src/asm/basino/basino -d /home/josh/src/asm/basino/rust-basino/src /home/josh/src/asm/basino/rust-basino/target/avr-atmega328p/debug/rust-basino.elf


target extended-remote :1234
# target remote :1234

# print demangled symbols
set print asm-demangle on

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# detect unhandled exceptions, hard faults and panics
# break DefaultHandler
# break HardFault
# break rust_begin_unwind
# # run the next few lines so the panic message is printed immediately
# # the number needs to be adjusted for your panic handler
# commands $bpnum
# next 4
# end

# *try* to stop at the user entry point (it might be gone due to inlining)
# break main
# load

# save command history
set history size 100000
set history save on

# start the process but immediately halt the processor
# stepi

# c
