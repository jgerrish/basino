# Arduino driver code
# To run in QEMU:
# qemu-system-avr -M uno -nographic -serial mon:stdio -bios basino_atmega328p
# To run in sim-avr:
# run_avr --mcu atmega328p basino_atmega328p
import board
import board / [serial, progmem]
import basino

# You can declare a whole set of types under a type statement.
# Very cool language feature.  Same with vars and a lot of other
# statement types.
type
  Stack {.packed.} = object
    stack: array[0..127, uint8]
    stack_top_sentinel: uint8

# Concatenate and send an arbitrary number of strings on the serial
# channel.
# This is actually an inefficient method, and shouldn't be used in a
# real AVR program.  The Ratel package provides ProgmemString strings,
# which store the string in program memory.  Usually there is much
# more program memory on Atmel devices.
# There are several improvements that could be made:
#   Overriding the & operator to operate on ProgmemStrings instead of
#   using varargs.  This includes supporting safe conversion to
#   cstring.
#   Creating a new type that supports multiple types of strings.
#   Doing automatic conversion of strings to ProgmemString when
#   appropriate.
proc send_strings(a: varargs[string]) =
  var tmp_str = ""

  for s in items(a):
    tmp_str &= s

  Serial.send cstring(tmp_str)

var stack = Stack()

Serial.init(9600.Hz)
Serial.send "Hello world\r\n"

Serial.send p"Adding 3 + 5: "
let res = basino_add(3, 5)
send_strings($res, "\r\n")

# Test the size of the stack
Serial.send p"Stack size: "
let stack_size = stack.stack.len()
var t_str = $stack_size
Serial.send cstring(t_str)
Serial.send "\r\n"

# Example of using a ProgmemString
# ProgmemStrings are stored in program memory, so take up less of the
# SRAM on limited memory AVR devices.
Serial.send p"Address adding 32767 + 32780"
let address_add_result_1 = basino_address_add(32767, 32780'u16)
send_strings(", result: ", $address_add_result_1, "\r\n")

Serial.send p"Address adding 16383 + 16383"
let address_add_result_2 = basino_address_add(16383, 16383)
send_strings(", result: ", $address_add_result_2, "\r\n")

let stack_addr = addr stack.stack
send_strings($cast[uint16](stack_addr), "\r\n")

let stack_top_sentinel_addr = addr stack.stack_top_sentinel
send_strings($cast[uint16](stack_top_sentinel_addr), "\r\n")

let res2 = basino_stack_init(stack_top_sentinel_addr, stack_addr, 128)
send_strings("Result of init: ", $res2, "\r\n")

Serial.send p"Value in basino_stack_bottom: "
let res3 = basino_get_basino_stack_bottom()
send_strings($res3, "\r\n")

Serial.send p"Value in basino_stack_top: "
let res4 = basino_get_basino_stack_top()
send_strings($res4, "\r\n")

Serial.send p"Value in basino_stack_size: "
let res5 = basino_get_basino_stack_size()
send_strings($res5, "\r\n")

let res6 = basino_stack_push(5)
send_strings("Result of push of 5: ", $res6, "\r\n")

let res7 = basino_stack_pop()
send_strings("Result of pop: ", $res7, "\r\n")

# Test popping from an empty stack
let res8 = basino_stack_pop()
send_strings("Result of empty stack pop: ", $res8, "\r\n")

# Test a series of pops and pushes
for i in countup(1'u8, 128'u8):
  let res = basino_stack_push(i)
  send_strings("Result of stack push ", $i, ": ", $res, "\r\n")

# This push should fail
let res9 = basino_stack_push(129)
send_strings("Result of stack push 129: ", $res9, "\r\n")

while true:
  power_down()
