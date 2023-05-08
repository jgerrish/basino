# Set of functions to use the basino BASIC library
#
# These functions don't test the API as thoroughly as the Rust code
# This is a known issue, and should probably be fixed.
# The error API is also out-of-sync, and before this code is used for
# production purposes that needs to be fixed.

# This result type is based on the result type from the Nim example
# parser combinator library: tests/misc/parsecomb.nim
# Any mistakes in Result and Error design are my own
# E.g. choosing to use another variant of ResultKind for rkSuccessNil
# instead of a type parameter to indicate None
# TODO: Define a set of error code enums
type
  ResultKind* = enum rkSuccess, rkSuccessNil, rkFailure, rkFailureNil
  Result*[T] = object
    case kind*: ResultKind
    of rkSuccessNil:
      nil
    of rkSuccess:
      val*: T
    of rkFailureNil:
      nil
    of rkFailure:
      error_code*: T

# when defined(Windows):
#   const libName* = "basino.dll"
# elif defined(Linux):
#   const libName* = "libbasino.so"
# elif defined(MacOsX):
#   const libName* = "libbasino.dylib"

const libName* = "libbasino.a"

{.link: libName.}

proc power_down*() {.importc: "power_down", cdecl.}

proc basino_add*(a: uint8, b: uint8): uint16 {.importc: "basino_add", cdecl.}

proc basino_address_add_c*(a: uint16, b: uint16,
    r: pointer): uint16 {.importc: "basino_address_add", cdecl.}

# Specifying the object type in the function declaration does not
# constrain the result constructors to the type, their type still
# needs to be explictly declared.
proc basino_address_add*(a: uint16, b: uint16): Result[uint16] =
  # Declare the result with var so it's mutable
  var address_add_result = 0'u8
  let address_add_result_addr = addr address_add_result

  let res = basino_address_add_c(a, b, address_add_result_addr)
  if address_add_result == 0:
    result = Result[uint16](kind: rkSuccess, val: res)
  else:
    result = Result[uint16](kind: rkFailureNil)

proc basino_stack_init_c*(stack: pointer, top: pointer, bottom: pointer): uint8 {.importc: "basino_stack_init", cdecl.}

proc basino_stack_init*(stack: pointer, top: pointer, bottom: pointer): Result[uint8] =
  let res = basino_stack_init_c(stack, top, bottom)
  if res == 0:
    result = Result[uint8](kind: rkSuccessNil)
  else:
    result = Result[uint8](kind: rkFailure, error_code: res)

# Test setting and getting the stack bottom, size, and stack start
# This also lets us test 16-bit return values
# The addresses should be consistent with a 128-byte change
proc basino_get_basino_stack_top*(stack: pointer): uint16 {.importc: "basino_get_basino_stack_top", cdecl.}

proc basino_get_basino_stack_top_sentinel*(stack: pointer): uint16 {.importc: "basino_get_basino_stack_top_sentinel", cdecl.}

proc basino_get_basino_stack_bottom*(stack: pointer): uint16 {.importc: "basino_get_basino_stack_bottom", cdecl.}

# proc basino_get_basino_stack_size*(stack: pointer): uint8 {.importc: "basino_get_basino_stack_size", cdecl.}

proc basino_stack_push_c*(stack: pointer, value: uint8): uint8 {.importc: "basino_stack_push", cdecl.}

# TODO: Still not ideal result type definition.  The unnecessary uint8
# isn't good.
# But it's better than simple integer return types.
proc basino_stack_push*(stack: pointer, value: uint8): Result[uint8] =
  let res = basino_stack_push_c(stack, value)
  if res == 0:
    result = Result[uint8](kind: rkSuccessNil)
  else:
    result = Result[uint8](kind: rkFailure, error_code: res)

proc basino_stack_pop_c*(stack: pointer, res: pointer): uint8 {.importc: "basino_stack_pop", cdecl.}

# Specifying the object type in the function declaration does not
# constrain the result constructors to the type, their type still
# needs to be explictly declared.
proc basino_stack_pop*(stack: pointer): Result[uint8] =
  # Declare the result with var so it's mutable
  var stack_pop_result = 0'u8
  let stack_pop_result_addr = addr stack_pop_result

  let res = basino_stack_pop_c(stack, stack_pop_result_addr)
  if stack_pop_result == 0:
    result = Result[uint8](kind: rkSuccess, val: res)
  else:
    result = Result[uint8](kind: rkFailure, error_code: stack_pop_result)






proc basino_queue_init_c*(queue: pointer, start: pointer, queue_end: pointer): uint8 {.importc: "basino_queue_init", cdecl.}

proc basino_queue_init*(queue: pointer, start: pointer, queue_end: pointer): Result[uint8] =
  let res = basino_queue_init_c(queue, start, queue_end)
  if res == 0:
    result = Result[uint8](kind: rkSuccessNil)
  else:
    result = Result[uint8](kind: rkFailure, error_code: res)

# Test setting and getting the stack bottom, size, and stack start
# This also lets us test 16-bit return values
# The addresses should be consistent with a 128-byte change
proc basino_queue_get_queue_start*(queue: pointer): uint16 {.importc: "basino_queue_get_queue_start", cdecl.}

proc basino_queue_get_queue_end*(queue: pointer): uint16 {.importc: "basino_queue_get_queue_end", cdecl.}

proc basino_queue_get_head*(queue: pointer): uint16 {.importc: "basino_queue_get_head", cdecl.}

proc basino_queue_get_last_head*(queue: pointer): uint16 {.importc: "basino_queue_get_end", cdecl.}

proc basino_queue_get_tail*(queue: pointer): uint16 {.importc: "basino_queue_get_tail", cdecl.}

proc basino_queue_put_c*(queue: pointer, value: uint8): uint8 {.importc: "basino_queue_put", cdecl.}

# TODO: Still not ideal result type definition.  The unnecessary uint8
# isn't good.
# But it's better than simple integer return types.
proc basino_queue_put*(queue: pointer, value: uint8): Result[uint8] =
  let res = basino_queue_put_c(queue, value)
  if res == 0:
    result = Result[uint8](kind: rkSuccessNil)
  else:
    result = Result[uint8](kind: rkFailure, error_code: res)

proc basino_queue_get_c*(stack: pointer, res: pointer): uint8 {.importc: "basino_queue_get", cdecl.}

proc basino_queue_get*(queue: pointer): Result[uint8] =
  # Declare the result with var so it's mutable
  var queue_get_result = 0'u8
  let queue_get_result_addr = addr queue_get_result

  let res = basino_queue_get_c(queue, queue_get_result_addr)
  if queue_get_result == 0:
    result = Result[uint8](kind: rkSuccess, val: res)
  else:
    result = Result[uint8](kind: rkFailure, error_code: queue_get_result)
