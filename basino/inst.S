.text

.globl basino_il_opcode_lb

.set BASINO_IL_IL_BYTE_CODE_OFFSET, 0
.set BASINO_IL_IL_BYTE_CODE_END_OFFSET, 2
.set BASINO_IL_IL_BYTE_CODE_PTR_OFFSET, 4
.set BASINO_IL_IL_INTERPRETER_STATE_OFFSET, 6
.set BASINO_IL_IL_QUEUE_OFFSET, 7
.set BASINO_IL_IL_STACK_OFFSET, 9

.set BASINO_IL_IL_STATE_RUNNING, 1
.set BASINO_IL_IL_STATE_STOPPED, 2
.set BASINO_IL_IL_STATE_EXECUTING_INSTRUCTION, 4

;; Execute the Push Literal Byte onto Stack (LB) opcode
;;
;; Parameters:
;;   Parameter 1: A pointer to the interpreter structure
;;
;; Returns:
;;   On success zero is returned.
;;   On failure, if interpreter structure is null, one is returned.
;;   On failure, if there was a stack overflow, two is returned.
;;   On failure, if the end-of-program was reached, three is returned.
;;   On failure, if run was attempted when the interpreter was stopped, four is returned.
;;   On failure, if there was an unknown error, five is returned.
basino_il_opcode_lb:
	;; Save our state
	push r14
	push r16
	push r22
	push r23
	push r25
	push r26
	push r27
	push r28
	push r29

	;; Setup the result code pointer
	ldi XL, lo8(basino_inst_result)
	ldi XH, hi8(basino_inst_result)

	;; movw r24, r18		; Copy r19:r18 -> r25:r24 (parameter one)
	movw r22, X		; Copy the address of basino_inst_result (parameter two)

	rcall basino_il_get_next_bytecode

	;; Get the result value from the result code pointer
	ldi XL, lo8(basino_inst_result)
	ldi XH, hi8(basino_inst_result)
	ld r14, X		; Get the result value

	ldi r16, 0x01
	cp r14, r16
	breq basino_il_opcode_lb_null_pointer

	ldi r16, 0x03
	cp r14, r16
	breq basino_il_opcode_lb_end_of_program

	ldi r16, 0x00
	cp r14, r16
	brne basino_il_opcode_lb_unknown_error

	;; Result code is 0x00, we can push the value on the stack
	; Set the second parameter to the item to push
	mov r22, r24

	;; Set the first parameter to the stack pointer
	ldd r24, Y+BASINO_IL_IL_STACK_OFFSET
	ldd r25, Y+BASINO_IL_IL_STACK_OFFSET+1

	;; Call the function
	rcall basino_stack_push

	ldi r16, 0x01
	cp r24, r16
	breq basino_il_opcode_lb_null_pointer

	ldi r16, 0x02
	cp r24, r16
	breq basino_il_opcode_lb_stack_overflow

	ldi r16, 0x00
	cp r24, r16
	breq basino_il_opcode_lb_successful

	rjmp basino_il_opcode_lb_unknown_error

basino_il_opcode_lb_successful:
	;; Set the return result to success
	ldi r24, 0x00
	rjmp basino_il_opcode_lb_end

basino_il_opcode_lb_null_pointer:
	;; Set the return result to NULL pointer error
	ldi r24, 0x01
	rjmp basino_il_opcode_lb_end

basino_il_opcode_lb_stack_overflow:
	;; Set the return result to stack overflow
	ldi r24, 0x02
	rjmp basino_il_opcode_lb_end

basino_il_opcode_lb_end_of_program:
	;; Set the return result to end of program
	ldi r24, 0x03
	rjmp basino_il_opcode_lb_end

basino_il_opcode_lb_stopped:
	;; Set the return result to stopped
	ldi r24, 0x04
	rjmp basino_il_opcode_lb_end

basino_il_opcode_lb_unknown_error:
	;; Set the return result to unknown error
	ldi r24, 0x05
	rjmp basino_il_opcode_lb_end

basino_il_opcode_lb_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r25
	pop r23
	pop r22
	pop r16
	pop r14

	ret

.section data

;; A variable to hold result codes from subroutine calls
basino_inst_result:
	.byte 0
