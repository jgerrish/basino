;; AVR stack implementation.
;;
;; A simple implementation of a stack on AVR devices.
;;
;; AVR supports pre-decrement and post-increment of index registers.  This informs
;; the stack design here.
;; First, we have a downward-growing stack.
;; We have a downward-growing stack, so the stack "top" should start one above
;; the true top.
;; We use a sentinel value at the top to do efficient equality
;; comparisons.

.set BASINO_STACK_SIZE,128

.text

.globl basino_stack_init, basino_stack_push, basino_stack_pop

.globl basino_stack_pop_low_byte_equal, basino_stack_pop_do_pop, basino_stack_pop_end, basino_stack_pop_underflow

;; Store the value onto the stack and decrement the top
;; On success zero is returned.
;; On failure, one is returned.
;;
;; AVR recommendations use variables instead of global registers.
;;
;; The LD instruction allows automatic post-incrementing or post-decrementing
;; of the X-Pointer Register.
basino_stack_push:
	push r20
	push r21
	push r26
	push r27
	push r28
	push r29

	ldi XL, lo8(basino_stack_top)
	ldi XH, hi8(basino_stack_top)
	;; Back up the index register so we can decrement the top later
	ld r28, X+
	ld r29, X

	ldi XL, lo8(basino_stack_bottom)
	ldi XH, hi8(basino_stack_bottom)
	ld r20, X+		; low byte
	ld r21, X		; high byte
	;; Test that we're not at the bottom of the stack
	;; Order of comparisons can actually matter here.
	;; Comparing the low-order byte first lets us avoid an
	;; unnecessary compare most of the time.
	cp r28, r20
	breq basino_stack_push_low_byte_equal
	;; We can always do an exact check, we shouldn't have to worry about going
	;; beyond the bottom.
	rjmp basino_stack_push_do_push

basino_stack_push_low_byte_equal:
	;; Compare the high-order byte now
	cp r29, r21
	breq basino_stack_push_stack_overflow
	rjmp basino_stack_push_do_push

basino_stack_push_do_push:
	;; Do the actual push of a value
	;; The byte to store is in the first parameter, r24
	;; In particular, AVR supports pre-decrement and post-increment
	;; We have a downward-growing stack, so the stack "top" should start one above
	;; the true top.
	st -Y, r24
	;; Store the new stack top
	ldi XL, lo8(basino_stack_top)
	ldi XH, hi8(basino_stack_top)

	st X+, r28
	st X, r29

	;; Return a successful result
	ldi r24, 0
	rjmp basino_stack_push_end

basino_stack_push_stack_overflow:
	;; Handle the case where we're at the bottom of the stack
	;; Return an unsuccessful result
	ldi r24, 1
	rjmp basino_stack_push_end

basino_stack_push_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r21
	pop r20

	ret

;; Load the value on the stack into the register, and increment the top afterwards
;;
;; The API of this is: pop expects a single parameter,
;; the address of the data to return.  It changes the value of this parameter
;; and returns a value of zero if it was successful.
;;
;; If there is an error trying to pop a value, the parameter value is
;; undefined and the error code is set to a non-zero value.
;;
;; It doesn't use a global errno like classic UNIX and POSIX
;; applications.
;;
;; The LD instruction allows automatic post-incrementing or post-decrementing
;; of the X-Pointer Register.
basino_stack_pop:
	push r20
	push r21
	push r26
	push r27
	push r28
	push r29

	;; Copy argument one (r24 and r25) to another register pair
	;; So we can return a result and use the argument
	movw r18, r24		; This copies over r24:45 -> r18:r19

	ldi XL, lo8(basino_stack_top)
	ldi XH, hi8(basino_stack_top)
	ld r28, X+
	ld r29, X

	ldi XL, lo8(basino_stack_top_sentinel)
	ldi XH, hi8(basino_stack_top_sentinel)
	ld r20, X+		; low byte
	ld r21, X		; high byte

	;; Test that we're not at the bottom of the stack
	;; Order of comparisons can actually matter here.
	;; Comparing the low-order byte first lets us avoid an
	;; unnecessary compare most of the time.
	;; One byte for a sentinel isn't that expensive, given the
	;; number of required stacks is small, for the BASIC
	;; use-case: so one sentinel.
	cp r28, r20
	breq basino_stack_pop_low_byte_equal
	;; We can always do an exact check, we shouldn't have to worry about
	;; getting out-of-sync.
	rjmp basino_stack_pop_do_pop

basino_stack_pop_low_byte_equal:
	;; Compare the high-order byte now
	cp r29, r21
	breq basino_stack_pop_underflow
	rjmp basino_stack_pop_do_pop

basino_stack_pop_do_pop:
	;; Do the actual push of a value
	;; The byte to store is in the first parameter, r24
	;; In particular, AVR supports pre-decrement and post-increment
	;; We have a downward-growing stack, so the stack "top" should start one above
	;; the true top.
	;; We also need to properly test and branch for that,
	;; either with a dummy space / sentinel (or stack_size - 1) (low-cost options),
	;; or additional non-equality checks.
	ld r24, Y+

	;; Store the new stack top
	ldi XL, lo8(basino_stack_top)
	ldi XH, hi8(basino_stack_top)
	st X+, r28
	st X, r29

	;; Return a successful result in the first parameter
	ldi r30, 0

	rjmp basino_stack_pop_end

basino_stack_pop_underflow:
	;; Handle the case where we're at the maximum top of the stack
	;; Return an unsuccessful result as the first parameter
	ldi r30, 1

	rjmp basino_stack_pop_end

basino_stack_pop_end:
	;; Set the return result (the first parameter)
	movw X, r18		; This copies r18:r19 to X
	st X, r30

	pop r29
	pop r28
	pop r27
	pop r26
	pop r21
	pop r20

	ret

;; Initialize the stack variables
;;
;; For reference, this code uses some of the X, Y and Z registers
;; These are 16-bit registers normally used for addressing
;; The X register is composed of R26 (low) and R27 (high)
;; The Y register is composed of R28 (low) and R29 (high)
;;   Y is a call-saved register in the GNU calling convention ABI
;;   "A function that uses such a registers must restore it's original content."
;; The Z register is composed of R30 (low) and R31 (high)
;; R2-R17 are also call-saved
;; Each register is also mapped to the first 32 locations in the data space,
;;
;; This function has two unsigned 16-bit parameters, and one unsigned 8-bit parameter:
;; argument one is the stack top address, an unsigned 16-bit integer, in r24 and r25
;;   The stack top starts at one more than the length of the stack, it
;;   points to a sentinel.
;; argument two is the stack bottom, an unsigned 16-bit integer, in r22 and r23
;; argument three is the stack size, an unsigned 8-bit integer, in r20
basino_stack_init:
	push r20
	push r21
	push r26
	push r27
	push r28
	push r29

	ldi XL, lo8(basino_stack_top)
	ldi XH, hi8(basino_stack_top)

	;; With st, the first operand is the address to store the data in
	st X+, r24
	st X, r25

	;; Set the stack top sentinel address too
	ldi XL, lo8(basino_stack_top_sentinel)
	ldi XH, hi8(basino_stack_top_sentinel)

	;; With st, the first operand is the address to store the data in
	st X+, r24
	st X, r25

	;; Set the basino_stack_bottom variable to the maximum bottom of the stack
	ldi XL, lo8(basino_stack_bottom)
	ldi XH, hi8(basino_stack_bottom)

	st X+, r22
	st X, r23

	;; Set the basino_stack_size variable to the stack size
	ldi XL, lo8(basino_stack_size)
	ldi XH, hi8(basino_stack_size)

	st X, r20

	ldi r24, 0

	pop r29
	pop r28
	pop r27
	pop r26
	pop r21
	pop r20

	ret

.globl basino_get_stack_bottom

basino_get_stack_bottom:
	ret

;; Variable printing functions
.globl basino_get_basino_stack_bottom, basino_get_basino_stack_top, basino_get_basino_stack_top_sentinel, basino_get_basino_stack_size, 

;; Return an unsigned short integer (two bytes) with the current stack bottom
basino_get_basino_stack_bottom:
	ldi XL, lo8(basino_stack_bottom)
	ldi XH, hi8(basino_stack_bottom)
	ld r24, X+		; low byte in short return
	ld r25, X		; high byte in short return

	ret

;; Return an unsigned short integer (two bytes) with the current basino_stack_top
basino_get_basino_stack_top:
	ldi XL, lo8(basino_stack_top)
	ldi XH, hi8(basino_stack_top)
	ld r24, X+
	ld r25, X

	ret

;; Return an unsigned short integer (two bytes) with the current stack top sentinel
basino_get_basino_stack_top_sentinel:
	ldi XL, lo8(basino_stack_top_sentinel)
	ldi XH, hi8(basino_stack_top_sentinel)
	ld r24, X+
	ld r25, X

	ret

;; Return an unsigned char (one byte) with the current stack size
basino_get_basino_stack_size:
	ldi XL, lo8(basino_stack_size)
	ldi XH, hi8(basino_stack_size)
	ld r24, X

	ret

;; stack metadata and variables

.section .data

.globl basino_stack_size, basino_stack_bottom, basino_stack_top, basino_stack, basino_stack_maximum_top

basino_stack:
	.space BASINO_STACK_SIZE
basino_stack_maximum_top:
basino_stack_top_sentinel:
	.short 0

basino_stack_size:
	.byte 0
basino_stack_bottom:
	.short 0
basino_stack_top:
	.short 0
