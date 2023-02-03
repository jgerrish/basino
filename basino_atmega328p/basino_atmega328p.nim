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
  Stack* {.packed.} = object
    # Allocate one extra array entry for the top sentinel
    data*: ptr UncheckedArray[uint8]
    top_sentinel*: ptr uint8
    bottom*: ptr uint8
    top*: ptr uint8

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

# Print out a test result
# TODO: This should be moved over to testament
proc send_test_result(test_result: bool, a: varargs[string]) =
  var tmp_str = ""

  if test_result:
    tmp_str &= "SUCCESS "
  else:
    tmp_str &= "FAILURE "

  for s in items(a):
    tmp_str &= s

  Serial.send cstring(tmp_str)


var stack = Stack()
var data: array[0..32, uint8]
stack.data = cast[ptr UncheckedArray[uint8]](data.unsafeAddr)

Serial.init(9600.Hz)

let res_add = basino_add(3, 5)
send_test_result(res_add == 8, "add of 3 + 5 should equal 8\r\n")

# Example of using a ProgmemString
# ProgmemStrings are stored in program memory, so take up less of the
# SRAM on limited memory AVR devices.
let address_add_result_1 = basino_address_add(32767, 32780'u16)
send_test_result(address_add_result_1.kind == rkFailure, "address add with carry should fail\r\n")

let address_add_result_2 = basino_address_add(16383, 16383)
send_test_result(address_add_result_2.kind == rkSuccess and address_add_result_2.val == 32766,
                 "address add without carry should work\r\n")

let stack_addr = addr stack
let stack_top_sentinel_addr = addr stack.data[32]

basino_stack_init(stack_addr, addr stack.data[32], addr stack.data[0], addr stack.data[32])

let res3 = basino_get_basino_stack_bottom(stack_addr)
send_test_result(res3 == cast[uint16](addr stack.data[0]), "stack bottom should be correct\r\n")

let res4 = basino_get_basino_stack_top(stack_addr)
send_test_result(res4 == cast[uint16](addr stack.data[32]), "stack top should be correct\r\n")


# Test the size of the stack
let stack_size = basino_get_basino_stack_top_sentinel(stack_addr) - basino_get_basino_stack_bottom(stack_addr)
var t_str = $stack_size
send_test_result(stack_size == 32, "stack size should be correct\r\n")

# Serial.send p"Value in basino_stack_size: "
# let res5 = basino_get_basino_stack_size(stack_addr)
# send_strings($res5, "\r\n")

var res = basino_stack_push(stack_addr, 5)
send_test_result(res.kind == rkSuccessNil, "Result of push of 5: ", $res, "\r\n")

res = basino_stack_pop(stack_addr)
send_test_result(res.kind == rkSuccess, "Result of pop: ", $res, "\r\n")

# Test popping from an empty stack
res = basino_stack_pop(stack_addr)
send_test_result(res.kind == rkFailure, "Result of empty stack pop: ", $res, "\r\n")

# Test a series of pops and pushes
for i in countup(1'u8, 32'u8):
  res = basino_stack_push(stack_addr, i)
  send_test_result(res.kind == rkSuccessNil, "Result of stack push ", $i, ": ", $res, "\r\n")

# This push should fail
res = basino_stack_push(stack_addr, 33)
send_test_result(res.kind == rkFailure, "Result of stack push 33: ", $res, "\r\n")

while true:
  power_down()
