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
;;
;; Most of the stack subroutines in this file assume that a stack structure has
;; already been created and allocated.  The subroutines do not own the stack structure,
;; The caller is responsible for creating and destroying that object.
;; There is an initialization function that can reset the stack to a new state, but
;; it doesn't allocate or own the object.
;;
;; AVR recommendations use variables instead of global registers.
;;
;; Below is a visual representation of the stack and pointers.
;; The size of the stack is four, data can be store in indices 0, 1, 2
;; and 3.
;; Location 4 is allocated and used as a sentinel top but not used for
;; data.
;;
;;     0   1   2   3   4
;;   +-------------------+
;;   |   |   |   |   |   |
;;   +-------------------+
;;     ^              ^ ^
;;     |              |  \
;;  bottom          top   -- sentinel top
;;
;;
;; After one push:
;;
;;     0   1   2   3   4
;;   +-------------------+
;;   |   |   |   | E |   |
;;   +-------------------+
;;     ^           ^    ^
;;     |           |     \
;;  bottom        top     -- sentinel top
;;

.text

.globl basino_stack_init, basino_stack_push, basino_stack_pop

.globl basino_stack_pop_low_byte_equal, basino_stack_pop_do_pop, basino_stack_pop_end, basino_stack_pop_underflow

;; These are defines that are used to calculate offsets into the stack
;; data structure.
.set BASINO_STACK_STACK_DATA_OFFSET, 0
.set BASINO_STACK_STACK_TOP_SENTINEL_OFFSET, 2
.set BASINO_STACK_STACK_BOTTOM_OFFSET, 4
.set BASINO_STACK_STACK_TOP_OFFSET, 6

;; Store the value onto the stack and decrement the top
;;
;; Parameters:
;; This function has one unsigned 16-bit parameters and one unsigned 8-bit parameter:
;;   parameter 1:
;;     Parameter one is a pointer to the start of the stack structure.
;;     It is placed in r24 and r25.
;;
;;   parameter 2:
;;     The value to push
;;
;; Returns:
;;   On success zero is returned.
;;   On failure, if the stack pointer is null, one is returned.
;;   On failure, if the stack is full, two is returned.
;;
basino_stack_push:
	push r16
	push r17
	push r18
	push r20
	push r21
	push r26
	push r27
	push r28
	push r29

basino_stack_push_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_stack_push_not_null_pointer
	cp r25, r16
	brne basino_stack_push_not_null_pointer
	rjmp basino_stack_push_null_pointer

basino_stack_push_not_null_pointer:
	;; Set up the stack structure
	movw Y, r24		; Copy r25:r24 -> Y

	;; Back up the index register so we can decrement the top later
	ldd r16, Y+BASINO_STACK_STACK_TOP_OFFSET
	ldd r17, Y+BASINO_STACK_STACK_TOP_OFFSET+1

	ldd r20, Y+BASINO_STACK_STACK_BOTTOM_OFFSET
	ldd r21, Y+BASINO_STACK_STACK_BOTTOM_OFFSET+1

	;; Test that we're not at the bottom of the stack
	;; Order of comparisons can actually matter here.
	;; Comparing the low-order byte first lets us avoid an
	;; unnecessary compare most of the time.
	cp r16, r20
	breq basino_stack_push_low_byte_equal
	;; We can always do an exact check, we shouldn't have to worry about going
	;; beyond the bottom.
	rjmp basino_stack_push_do_push

basino_stack_push_low_byte_equal:
	;; Compare the high-order byte now
	cp r17, r21
	breq basino_stack_push_stack_overflow
	rjmp basino_stack_push_do_push

basino_stack_push_do_push:
	;; Do the actual push of a value
	;; The byte to store is in the second parameter, r22
	;; In particular, AVR supports pre-decrement and post-increment
	;; We have a downward-growing stack, so the stack "top" should start one above
	;; the true top.
	ldd r26, Y+BASINO_STACK_STACK_TOP_OFFSET
	ldd r27, Y+BASINO_STACK_STACK_TOP_OFFSET+1

	st -X, r22

	;; Store the new stack top
	std Y+BASINO_STACK_STACK_TOP_OFFSET, r26
	std Y+BASINO_STACK_STACK_TOP_OFFSET+1, r27

	;; Return a successful result
	ldi r25, 0
	ldi r24, 0
	rjmp basino_stack_push_end

basino_stack_push_null_pointer:
	ldi r25, 0
	ldi r24, 1
	rjmp basino_stack_push_end

basino_stack_push_stack_overflow:
	;; Handle the case where we're at the bottom of the stack
	;; Return an unsuccessful result
	ldi r25, 0
	ldi r24, 2
	rjmp basino_stack_push_end

basino_stack_push_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r21
	pop r20
	pop r18
	pop r17
	pop r16

	ret

;; Pop a value from the stack, and increment the top afterwards
;;
;; The API of this is: pop expects two parameters,
;; A pointer to the stack and a pointer to a return result variable.
;; It changes the value of this parameter to zero if it was
;; successful.
;;
;; If there is an error trying to pop a value, the return value is
;; undefined and the error code is set to a non-zero value.
;;
;; Parameters:
;; This function has one unsigned 16-bit parameters and one unsigned 8-bit parameter:
;;   parameter 1:
;;     Parameter one is a pointer to the start of the stack structure.
;;     It is placed in r24 and r25.
;;
;;   parameter 2:
;;     A pointer to a result code variable.
;;     Sets the variable to zero if the pop was successful.
;;     Sets the variable to one if there is a null stack pointer.
;;     Sets the variable to two if there is a stack underflow.
;;
basino_stack_pop:
	push r16
	push r17
	push r20
	push r21
	push r26
	push r27
	push r28
	push r29
	push r30

basino_stack_pop_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_stack_pop_not_null_pointer
	cp r25, r16
	brne basino_stack_pop_not_null_pointer
	rjmp basino_stack_pop_null_pointer

basino_stack_pop_not_null_pointer:
	;; Set up the stack structure
	movw Y, r24		; Copy r25:r24 -> Y

	;; Copy argument two (r22 and r23) to another register pair
	;; So we can return a result and use the argument
	movw r18, r22		; This copies over r23:22 -> r19:r18

	ldd r16, Y+BASINO_STACK_STACK_TOP_OFFSET
	ldd r17, Y+BASINO_STACK_STACK_TOP_OFFSET+1

	ldd r20, Y+BASINO_STACK_STACK_TOP_SENTINEL_OFFSET
	ldd r21, Y+BASINO_STACK_STACK_TOP_SENTINEL_OFFSET+1

	;; Test that we're not at the bottom of the stack
	;; Order of comparisons can actually matter here.
	;; Comparing the low-order byte first lets us avoid an
	;; unnecessary compare most of the time.
	;; One byte for a sentinel isn't that expensive, given the
	;; number of required stacks is small, for the BASIC
	;; use-case: so one sentinel.
	cp r16, r20
	breq basino_stack_pop_low_byte_equal
	;; We can always do an exact check, we shouldn't have to worry about
	;; getting out-of-sync.
	rjmp basino_stack_pop_do_pop

basino_stack_pop_low_byte_equal:
	;; Compare the high-order byte now
	cp r17, r21
	breq basino_stack_pop_underflow
	rjmp basino_stack_pop_do_pop

basino_stack_pop_do_pop:
	;; Do the actual pop of a value
	;; The byte to store is in the first parameter, r24
	;; In particular, AVR supports pre-decrement and post-increment
	;; We have a downward-growing stack, so the stack "top" should start one above
	;; the true top.
	;; We also need to properly test and branch for that,
	;; either with a dummy space / sentinel (or stack_size - 1) (low-cost options),
	;; or additional non-equality checks.
	ldd r26, Y+BASINO_STACK_STACK_TOP_OFFSET
	ldd r27, Y+BASINO_STACK_STACK_TOP_OFFSET+1

	ld r24, X+

	;; Store the new stack top
	std Y+BASINO_STACK_STACK_TOP_OFFSET, r26
	std Y+BASINO_STACK_STACK_TOP_OFFSET+1, r27

	;; Return a successful result in the first parameter
	ldi r30, 0

	rjmp basino_stack_pop_end

basino_stack_pop_null_pointer:
	ldi r30, 1
	rjmp basino_stack_pop_end

basino_stack_pop_underflow:
	;; Handle the case where we're at the maximum top of the stack
	;; Return an unsuccessful result as the first parameter
	ldi r30, 2

	rjmp basino_stack_pop_end

basino_stack_pop_end:
	;; Set the return result (the first parameter)
	movw X, r22		; This copies r23:r22 (second argument) to X
	st X, r30

	pop r30
	pop r29
	pop r28
	pop r27
	pop r26
	pop r21
	pop r20
	pop r17
	pop r16

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
;; Returns an 8-bit result code
;;
;; Parameters:
;; This function has three unsigned 16-bit parameters, and one unsigned 8-bit parameter:
;;   parameter 1:
;;     Parameter one is a pointer to the start of the stack structure.
;;     It is placed in r24 and r25.
;;
;;   parameter 2:
;;     Parameter two is the stack maximum top address or stack top sentinel,
;;     an unsigned 16-bit integer, in r22 and r23
;;     The stack top starts at one more than the length of the stack, it
;;     points to a sentinel.
;;
;;   parameter 3:
;;     argument three is the stack bottom, an unsigned 16-bit integer, in r20 and r21
;;
;; Returns:
;;   Zero if the subroutine is successful
;;   One if the stack pointer parameter is a null pointer
;;   Two if the stack bottom was above the top sentinel
;;
basino_stack_init:
	push r15
	push r16
	push r17
	push r20
	push r21
	push r26
	push r27
	push r28
	push r29

basino_stack_init_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_stack_init_top_bottom_check
	cp r25, r16
	brne basino_stack_init_top_bottom_check
	rjmp basino_stack_init_null_pointer

basino_stack_init_top_bottom_check:
	movw r16, r22		; Copy r23:r22 -> r17:r16

	;; Now check that the stack top is above the stack bottom
	;;
	;; Equal bottom and top means that the stack size is zero.
	;; This could technically be possible, even if it's not
	;; useful.

	push r22
	push r23
	push r24
	push r25

	movw r24, r22
	movw r22, r20
	rcall basino_gt
	cpi r24, 0

	pop r25
	pop r24
	pop r23
	pop r22

	breq basino_stack_init_top_lt_eq_bottom
	rjmp basino_stack_init_do_init

basino_stack_init_do_init:
	movw Y, r24		; Copy r25:r24 -> Y

	;; Set the pointers in the stack structure to actual locations
	;; in the stack.  This assumes a normal array that is
	;; continuous.
	;; Note: This doesn't do post-increment or pre-decrement on
	;; the pointer.
	std Y+BASINO_STACK_STACK_TOP_OFFSET, r22
	std Y+BASINO_STACK_STACK_TOP_OFFSET+1, r23

	std Y+BASINO_STACK_STACK_BOTTOM_OFFSET, r20
	std Y+BASINO_STACK_STACK_BOTTOM_OFFSET+1, r21

	std Y+BASINO_STACK_STACK_TOP_SENTINEL_OFFSET, r22
	std Y+BASINO_STACK_STACK_TOP_SENTINEL_OFFSET+1, r23

	rjmp basino_stack_init_successful

basino_stack_init_null_pointer:
	ldi r25, 0
	ldi r24, 1
	rjmp basino_stack_init_end

basino_stack_init_top_lt_eq_bottom:
	ldi r25, 0
	ldi r24, 2
	rjmp basino_stack_init_end

basino_stack_init_successful:
	ldi r25, 0
	ldi r24, 0
	rjmp basino_stack_init_end

basino_stack_init_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r21
	pop r20
	pop r17
	pop r16
	pop r15

	ret

;; Variable printing functions
.globl basino_get_basino_stack_bottom, basino_get_basino_stack_top, basino_get_basino_stack_top_sentinel

;; Return an unsigned short integer (two bytes) with the current stack bottom
basino_get_basino_stack_bottom:
	push r26
	push r27
	push r28
	push r29

	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_STACK_STACK_BOTTOM_OFFSET
	ldd r25, Y+BASINO_STACK_STACK_BOTTOM_OFFSET+1

	pop r29
	pop r28
	pop r27
	pop r26

	ret

;; Return an unsigned short integer (two bytes) with the current basino_stack_top
basino_get_basino_stack_top:
	push r26
	push r27
	push r28
	push r29

	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_STACK_STACK_TOP_OFFSET
	ldd r25, Y+BASINO_STACK_STACK_TOP_OFFSET+1

	pop r29
	pop r28
	pop r27
	pop r26

	ret

;; Return an unsigned short integer (two bytes) with the current stack top sentinel
basino_get_basino_stack_top_sentinel:
	push r26
	push r27
	push r28
	push r29

	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_STACK_STACK_TOP_SENTINEL_OFFSET
	ldd r25, Y+BASINO_STACK_STACK_TOP_SENTINEL_OFFSET+1

	pop r29
	pop r28
	pop r27
	pop r26

	ret

;; Return an unsigned short (two bytes) with the current stack size
;;
;; Removed in latest version
;;
;; This can be re-implemented with basino_get_basino_stack_top_sentienl
;; and basino_get_basino_bottom.
;;
;; basino_get_basino_stack_size:
;; 	push r28
;; 	push r29

;; 	movw Y, r24		; Copy r25:r24 -> Y

;; 	ldi r25, 0
;; 	ldd r24, Y+BASINO_STACK_STACK_SIZE_OFFSET

;; 	pop r29
;; 	pop r28

;; 	ret


