;;
;; Tiny BASIC Interpretive Language (IL) VM
;;

.text

.globl basino_il_init, basino_il_run, basino_il_exec, basino_il_get_next_bytecode

.set BASINO_IL_IL_BYTE_CODE_OFFSET, 0
.set BASINO_IL_IL_BYTE_CODE_END_OFFSET, 2
.set BASINO_IL_IL_BYTE_CODE_PTR_OFFSET, 4
.set BASINO_IL_IL_INTERPRETER_STATE_OFFSET, 6
.set BASINO_IL_IL_QUEUE_OFFSET, 7
.set BASINO_IL_IL_STACK_OFFSET, 9

.set BASINO_IL_IL_STATE_RUNNING, 1
.set BASINO_IL_IL_STATE_STOPPED, 2
.set BASINO_IL_IL_STATE_EXECUTING_INSTRUCTION, 4

;; Initialized the Interpretive Language (IL) VM
;;
;; Parameters:
;;   Parameter 1: A pointer to the interpreter structure
;;   Parameter 2: A pointer to the start of the byte code
;;   Parameter 3: A pointer to the length of the byte code array
;;   Parameter 4: A pointer to the input queue
;;   Parameter 5: A pointer to the computation stack
;;
;; Returns:
;;   On success zero is returned.
;;   On failure, if interpreter structure is null, one is returned.
;;   On failure, if there are invalid arguments, two is returned.
basino_il_init:
	push r12
	push r13
	push r14
	push r16
	push r17
	push r18
	push r19
	push r20
	push r21
	push r22
	push r26
	push r27
	push r28
	push r29

basino_il_init_null_pointer_check:
	movw r12, r18		; Copy r19:r18 to r13:r12
	;; Check that the first parameter is not a null pointer
	ldi r18, 0x00
	cp r24, r18
	brne basino_il_init_len_check
	cp r25, r18
	brne basino_il_init_len_check
	rjmp basino_il_init_null_pointer

basino_il_init_len_check:
	;; The length should be at least one
	ldi r18, 0x00
	cp r20, r18
	brne basino_il_init_do_init
	cp r21, r18
	brne basino_il_init_do_init
	rjmp basino_il_init_invalid_arguments

basino_il_init_do_init:
	movw r18, r12		; Copy r13:r12 to r19:r18

	movw Y, r24		; Copy r25:r24 -> Y

	;; Set the pointers in the il structure to actual locations
	;; in the il.  This assumes a normal array that is
	;; continuous.
	std Y+BASINO_IL_IL_BYTE_CODE_OFFSET, r22
	std Y+BASINO_IL_IL_BYTE_CODE_OFFSET+1, r23

	;; Set the current pointer to the beginning of the array
	std Y+BASINO_IL_IL_BYTE_CODE_PTR_OFFSET, r22
	std Y+BASINO_IL_IL_BYTE_CODE_PTR_OFFSET+1, r23

	std Y+BASINO_IL_IL_QUEUE_OFFSET, r18
	std Y+BASINO_IL_IL_QUEUE_OFFSET+1, r19

	std Y+BASINO_IL_IL_STACK_OFFSET, r16
	std Y+BASINO_IL_IL_STACK_OFFSET+1, r17

	ldi r16, BASINO_IL_IL_STATE_RUNNING
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r16

	;; Initialize the end of the array by adding the length to the
	;; array start minus one
	push r20
	push r21
	push r22
	push r23
	push r24
	push r25
	push r26
	push r27

	;; Setup parameter one
	movw r24, r22	     ; Copy the start of the array to r25:r24

	;; Subtract one from the length
	movw r26, r20	     ; Copy the length of the array to r27:r26
	sbiw r26, 0x01	     ; 16-bit subtract 1 (r27:r26 = r27:r26 - 1)

	;; Setup parameter two
	movw r22, r26	     ; Copy r27:r26 to r23:r22

	;; Setup parameter 3, the result code pointer
	ldi XL, lo8(basino_il_result)
	ldi XH, hi8(basino_il_result)

	movw r20, X

	rcall basino_address_add

	;; Check the return result
	ldi XL, lo8(basino_il_result)
	ldi XH, hi8(basino_il_result)
	ld r14, X		; Get the result value

	ldi r16, 0x01
	cp r14, r16
	breq basino_il_init_address_add_invalid_arguments

	std Y+BASINO_IL_IL_BYTE_CODE_END_OFFSET, r24
	std Y+BASINO_IL_IL_BYTE_CODE_END_OFFSET+1, r25

	pop r27
	pop r26
	pop r25
	pop r24
	pop r23
	pop r22
	pop r21
	pop r20

	rjmp basino_il_init_successful

basino_il_init_null_pointer:
	;; Set the return result to NULL pointer error
	ldi r24, 0x01
	rjmp basino_il_init_end

basino_il_init_invalid_arguments:
	;; Set the return result to an invalid arguments error
	ldi r24, 0x02
	rjmp basino_il_init_end

basino_il_init_address_add_invalid_arguments:
	;; Set the return result to an invalid arguments error
	pop r27
	pop r26
	pop r25
	pop r24
	pop r23
	pop r22
	pop r21
	pop r20

	ldi r24, 0x02
	rjmp basino_il_init_end


basino_il_init_successful:
	;; Set the return result to successful
	ldi r24, 0x00
	rjmp basino_il_init_end

basino_il_init_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r22
	pop r21
	pop r20
	pop r19
	pop r18
	pop r17
	pop r16
	pop r14
	pop r13
	pop r12

	ret


;; Get the next byte from the bytecode stream
;;
;; Parameters:
;;   Parameter 1: A pointer to the interpreter structure
;;   Parameter 2:
;;     A pointer to a result code variable.
;;     Sets the variable to zero if the operation was successful.
;;     Sets the variable to one if there is a null interpreter pointer.
;;     Sets the variable to three if end-of-program was reached
;;
;; Returns:
;;   The next byte in the bytecode stream
basino_il_get_next_bytecode:
	push r14
	push r15
	push r16
	push r17
	push r18
	push r19
	push r26
	push r27
	push r28
	push r29
	push r30

basino_il_get_next_bytecode_null_pointer_check:
	;; Check that the first parameter is not a null pointer
	ldi r16, 0x00
	cp r24, r16
	brne basino_il_get_next_bytecode_do_get_next_byte_code
	cp r25, r16
	brne basino_il_get_next_bytecode_do_get_next_byte_code
	rjmp basino_il_get_next_bytecode_null_pointer

basino_il_get_next_bytecode_do_get_next_byte_code:
	movw Y, r24		; Copy r25:r24 -> Y

	;; Get the current byte code position
	ldd r26, Y+BASINO_IL_IL_BYTE_CODE_PTR_OFFSET
	ldd r27, Y+BASINO_IL_IL_BYTE_CODE_PTR_OFFSET+1

	;; Check if the current pointer is at the end of the bytecode
	;; array.
	ldd r18, Y+BASINO_IL_IL_BYTE_CODE_END_OFFSET
	ldd r19, Y+BASINO_IL_IL_BYTE_CODE_END_OFFSET+1

	cp r18, r26
	brne basino_il_get_next_bytecode_do_get_next_byte_code_do
	cp r19, r27
	brne basino_il_get_next_bytecode_do_get_next_byte_code_do
	;; If the current pointer and end pointer are equal, exit
	;; with end-of-program error
	rjmp basino_il_get_next_bytecode_end_of_program

basino_il_get_next_bytecode_do_get_next_byte_code_do:
	ld r14, X+		; Get the item in the queue and advance

	;; Store the new byte code pointer
	std Y+BASINO_IL_IL_BYTE_CODE_PTR_OFFSET, r26
	std Y+BASINO_IL_IL_BYTE_CODE_PTR_OFFSET+1, r27

	rjmp basino_il_get_next_bytecode_successful

basino_il_get_next_bytecode_successful:
	;; Set the return result to successful
	ldi r30, 0x00
	rjmp basino_il_get_next_bytecode_end

basino_il_get_next_bytecode_null_pointer:
	;; Set the return result to NULL pointer error
	ldi r30, 0x01
	rjmp basino_il_get_next_bytecode_end

basino_il_get_next_bytecode_end_of_program:
	;; Set the interpreter to stopped
	ldi r16, BASINO_IL_IL_STATE_STOPPED
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r16

	;; Set the return result to end-of-program error
	ldi r30, 0x03
	rjmp basino_il_get_next_bytecode_end

basino_il_get_next_bytecode_end:
	movw X, r22		; Copy r23:r22 (second parameter) to X
	st X, r30
	mov r24, r14		; Copy over the item to r24 for return

	pop r30
	pop r29
	pop r28
	pop r27
	pop r26
	pop r19
	pop r18
	pop r17
	pop r16
	pop r15
	pop r14

	ret


;;
;; Run an Interpretive Language (IL) program
;;
;; Parameters:
;;   Parameter 1: A pointer to the interpreter structure
;;
;; Returns:
;;   On success zero is returned.
;;   On failure, if interpreter structure is null, one is returned.
;;   On failure, if the instruction wasn't found, three is returned.
;;   On failure, if run was attempted when the interpreter was stopped, five is returned.
;;   On failure, if there was an unknown error, six is returned.
basino_il_run:
	push r14
	push r15
	push r16
	push r17
	push r18
	push r19
	push r22
	push r23
	push r25
	push r26
	push r27
	push r28
	push r29

	;; Save the interpreter pointer
	movw r18, r24

basino_il_run_null_pointer_check:
	;; Check that the first parameter is not a null pointer
	ldi r16, 0x00
	cp r24, r16
	brne basino_il_run_do_run
	cp r25, r16
	brne basino_il_run_do_run
	rjmp basino_il_run_null_pointer

basino_il_run_do_run:
	;; This is the main interpreter run-time loop
	;; We fetch the next instruction opcode and execute the instruction

	;; Set up the stack structure
	movw Y, r18		; Copy r19:r18 -> Y

	;; Check if the interpreter state is stopped
	ldd r15, Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET
	ldi r17, BASINO_IL_IL_STATE_STOPPED
	cp r15, r17

	;; If the interpreter is stopped, exit with a successful status code
	breq basino_il_run_stopped

	;; Else, get the next instruction and execute it
	;; First, set our state to executing an instruction
	ldi r17, BASINO_IL_IL_STATE_EXECUTING_INSTRUCTION
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r17

	;; Setup the result code pointer
	ldi XL, lo8(basino_il_result)
	ldi XH, hi8(basino_il_result)

	movw r24, r18		; Copy r19:r18 -> r25:r24 (parameter one)
	movw r22, X		; Copy the address of basino_il_result (parameter two)

	rcall basino_il_get_next_bytecode

	;; Get the result value from the result code pointer
	ldi XL, lo8(basino_il_result)
	ldi XH, hi8(basino_il_result)
	ld r14, X		; Get the result value

	ldi r16, 0x01
	cp r14, r16
	breq basino_il_run_null_pointer

	ldi r16, 0x03
	cp r14, r16
	breq basino_il_run_end_of_program

	ldi r16, 0x00
	cp r14, r16

	;; Successful result, execute the instruction
	breq basino_il_run_check_eop

	;; Unsuccessful result, return an error
	rjmp basino_il_run_unknown_error

basino_il_run_check_eop:
	;; See if we found a zero byte-code while not exectuing an
	;; instruction
	ldd r15, Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET
	ldi r17, BASINO_IL_IL_STATE_EXECUTING_INSTRUCTION

	cp r15, r17

	;; Not equal, execute the instruction
	brne basino_il_run_do_exec

	;; If we're executing an instruction, see if the next
	;; bytecode is 0x00.  That indicates end of program
	ldi r17, 0x00
	cp r24, r17
	breq basino_il_run_end_of_program

	;; else execute the bytecode
	rjmp basino_il_run_do_exec

basino_il_run_do_exec:
	;; Execute the bytecode

	;; Set the second parameter to the instruction bytecode
	mov r22, r24

	;; Restore the interpreter pointer
	movw r24, Y	; Copy Y -> r25:r24

	;; Execute the instruction
	rcall basino_il_exec

	;; Check the return code
	ldi r16, 0x00
	cp r24, r16
	breq basino_il_run_do_run

	ldi r16, 0x01
	cp r24, r16
	breq basino_il_run_null_pointer

	ldi r16, 0x03
	cp r24, r16
	breq basino_il_run_instruction_not_found

	;; Jump back to the start of the runtime loop
	rjmp basino_il_run_do_run

basino_il_run_successful:
	;; Set our state to stopped
	ldi r17, BASINO_IL_IL_STATE_STOPPED
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r17

	;; Set the return result to successful
	ldi r24, 0x00
	rjmp basino_il_run_end

basino_il_run_null_pointer:
	;; Set our state to stopped
	ldi r17, BASINO_IL_IL_STATE_STOPPED
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r17

	;; Set the return result to NULL pointer error
	ldi r24, 0x01
	rjmp basino_il_run_end

basino_il_run_instruction_not_found:
	;; Set our state to stopped
	ldi r17, BASINO_IL_IL_STATE_STOPPED
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r17

	;; Set the return result to instruction not found
	ldi r24, 0x03
	rjmp basino_il_run_end

basino_il_run_end_of_program:
	;; Set our state to stopped
	ldi r17, BASINO_IL_IL_STATE_STOPPED
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r17

	;; Set the return result to success
	ldi r24, 0x00
	rjmp basino_il_run_end

basino_il_run_stopped:
	;; Set the return result to stopped
	ldi r24, 0x05
	rjmp basino_il_run_end

basino_il_run_unknown_error:
	;; Set our state to stopped
	ldi r17, BASINO_IL_IL_STATE_STOPPED
	std Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET, r17

	ldi r24, 0x06
	rjmp basino_il_run_end

basino_il_run_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r25
	pop r23
	pop r22
	pop r19
	pop r18
	pop r17
	pop r16
	pop r15
	pop r14

	ret


;; Execute a single Interpretive Language (IL) instruction
;;
;; Parameters:
;;   Parameter 1: A pointer to the interpreter structure
;;   Parameter 2: The instruction to execute
;;
;; Returns:
;;   On success zero is returned.
;;   On failure, if interpreter structure is null, one is returned.
;;   On failure, if the instruction wasn't found, three is returned.
;;   On failure, if run was attempted when the interpreter was stopped, four is returned.
;;   On failure, if there was an unknown error, six is returned.
basino_il_exec:
	push r14
	push r16
	push r17
	push r18
	push r19
	push r22
	push r23
	push r25
	push r26
	push r27
	push r28
	push r29

basino_il_exec_null_pointer_check:
	;; Check that the first parameter is not a null pointer
	ldi r16, 0x00
	cp r24, r16
	brne basino_il_exec_running_check
	cp r25, r16
	brne basino_il_exec_running_check
	rjmp basino_il_exec_null_pointer

basino_il_exec_running_check:
	movw Y, r24		; Copy r25:r24 -> Y

	;; Check if the interpreter state is stopped
	ldd r16, Y+BASINO_IL_IL_INTERPRETER_STATE_OFFSET
	ldi r17, BASINO_IL_IL_STATE_STOPPED
	cp r16, r17

	;; If the interpreter is stopped, exit with a successful status code
	breq basino_il_exec_stopped

	rjmp basino_il_exec_do_exec

basino_il_exec_do_exec:
	;; Maybe make more use of opcode structure

	;; NO 0x08 No Operation
	ldi r16, 0x08
	cp r22, r16
	breq basino_il_exec_opcode_no

	;; LB 0x09nn Push Literal Byte onto Stack
	ldi r16, 0x09
	cp r22, r16
	breq basino_il_exec_opcode_lb

	rjmp basino_il_exec_instruction_not_found

basino_il_exec_successful:
	;; Set the return result to successful
	ldi r24, 0x00
	rjmp basino_il_exec_end

basino_il_exec_null_pointer:
	;; Set the return result to NULL pointer error
	ldi r24, 0x01
	rjmp basino_il_exec_end

basino_il_exec_instruction_not_found:
	;; Set the return result to instruction not found
	ldi r24, 0x03
	rjmp basino_il_exec_end

basino_il_exec_stopped:
	;; Set the return result to instruction not found
	ldi r24, 0x04
	rjmp basino_il_exec_end

basino_il_exec_stack_overflow:
	;; Set the return result to stack overflow
	ldi r24, 0x05
	rjmp basino_il_exec_end

basino_il_exec_unknown_error:
	;; Set the return result to unknown error
	ldi r24, 0x06
	rjmp basino_il_exec_end


basino_il_exec_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r25
	pop r23
	pop r22
	pop r19
	pop r18
	pop r17
	pop r16
	pop r14

	ret


;; Instruction jump section

;; Execute the No Operation (NO) opcode
;; Does nothing
basino_il_exec_opcode_no:
	rjmp basino_il_exec_successful

basino_il_exec_opcode_lb:
	rcall basino_il_opcode_lb

	ldi r16, 0x01
	cp r24, r16
	breq basino_il_exec_null_pointer

	ldi r16, 0x02
	cp r24, r16
	breq basino_il_exec_stack_overflow

	ldi r16, 0x00
	cp r24, r16
	breq basino_il_exec_successful

	rjmp basino_il_exec_unknown_error

.section data

;; A variable to hold result codes from subroutine calls
basino_il_result:
	.byte 0
