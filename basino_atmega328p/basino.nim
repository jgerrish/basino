# Set of functions to use the basino BASIC library

# This result type is based on the result type from the Nim example
# parser combinator library: tests/misc/parsecomb.nim
# Any mistakes in Result and Error design are my own
# E.g. choosing to use another variant of ResultKind for rkSuccessNil
# instead of a type parameter to indicate None
# TODO: Define a set of error code enums
type
  ResultKind* = enum rkSuccess, rkSuccessNil, rkFailure
  Result*[T] = object
    case kind*: ResultKind
    of rkSuccessNil:
      nil
    of rkSuccess:
      val: T
    of rkFailure:
      nil

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
    result = Result[uint16](kind: rkFailure)

proc basino_stack_init_c*(stack_top: pointer, stack_bottom: pointer,
    stack_size: uint8): uint8 {.importc: "basino_stack_init", cdecl.}

proc basino_stack_init*(stack_top: pointer, stack_bottom: pointer,
                       stack_size: uint8): Result[uint8] =
  let res = basino_stack_init_c(stack_top, stack_bottom, stack_size)
  if res == 0:
    result = Result[uint8](kind: rkSuccessNil)
  else:
    result = Result[uint8](kind: rkFailure)

# Test setting and getting the stack bottom, size, and stack start
# This also lets us test 16-bit return values
# The addresses should be consistent with a 128-byte change
proc basino_get_basino_stack_top*(): uint16 {.importc: "basino_get_basino_stack_top", cdecl.}

proc basino_get_basino_stack_bottom*(): uint16 {.importc: "basino_get_basino_stack_bottom", cdecl.}

proc basino_get_basino_stack_size*(): uint8 {.importc: "basino_get_basino_stack_size", cdecl.}

proc basino_stack_push_c*(value: uint8): uint8 {.importc: "basino_stack_push", cdecl.}

# TODO: Still not ideal result type definition.  The unnecessary uint8
# isn't good.
# But it's better than simple integer return types.
proc basino_stack_push*(value: uint8): Result[uint8] =
  let res = basino_stack_push_c(value)
  if res == 0:
    result = Result[uint8](kind: rkSuccessNil)
  else:
    result = Result[uint8](kind: rkFailure)

proc basino_stack_pop_c*(res: pointer): uint8 {.importc: "basino_stack_pop", cdecl.}

# Specifying the object type in the function declaration does not
# constrain the result constructors to the type, their type still
# needs to be explictly declared.
proc basino_stack_pop*(): Result[uint8] =
  # Declare the result with var so it's mutable
  var stack_pop_result = 0'u8
  let stack_pop_result_addr = addr stack_pop_result

  let res = basino_stack_pop_c(stack_pop_result_addr)
  if stack_pop_result == 0:
    result = Result[uint8](kind: rkSuccess, val: res)
  else:
    result = Result[uint8](kind: rkFailure)
