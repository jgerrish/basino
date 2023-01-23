;; This is a Tiny BASIC implementation targeting embedded AVR
;; devices.
;; Right now, it only includes a stack data structure.
;;
;; Some notes about calling conventions:
;;
;; The default NIM ABI differs from the one in
;; "AVR034: Mixing C and Assembly Code with IAR Embedded Workbench for AVR"
;; Atmel document 1234B-AVR-04/03
;;
;; Oregon State ECE473 has a lecture with a good diagram showing the
;; calling convention:
;; https://web.engr.oregonstate.edu/~traylor/ece473/lectures/assembly.pdf
;;
;; r24 has parameter one
;; r22 has parameter two
;;
;; registers expected from 1234B-AVR-04/03: R16 and R20
;; The calling convention matches the one described in
;; "Application Binary Interface and implementation defined
;;  behaviour of avr-gcc."
;; In particular using the next lowest even number, starting
;; at r26.
;;
;; For reference, this code uses some of the X, Y and Z registers
;; These are 16-bit registers normally used for addressing
;; X and Y are for accessing values in the data memory
;; Z is for accessing values in the program memory
;; The X register is composed of R26 (low) and R27 (high)
;; The Y register is composed of R28 (low) and R29 (high)
;;   Y is a call-saved register in the GNU calling convention ABI
;;   "A function that uses such a registers must restore it's original content."
;; The Z register is composed of R30 (low) and R31 (high)
;; R2-R17 are also call-saved
;; R18-R27, R30, R31 are call-used registers.  They can be clobbered by functions but not
;; ISRs (Interrupt Service Routines / Interrupt Handlers)
;; Some additional registers are saved that probably don't need to be saved.
;; Each register is also mapped to the first 32 locations in the data space,

.text
.globl basino_add, basino_address_add, basino_start, power_down

;; Initialize the system
;; Currently, this doesn't set up anything.
;; The individual data structures have their own initialization
;; routines.
basino_start:
	ret

;; Add two unsigned 8-bit numbers and return the result.
;; Expects two char or byte-size arguments, in r24 and r22.
;; Returns the result as a short in r24 and r25
;; This was used as a basic test function to test the Nim and Rust
;; drivers.
basino_add:
	add r24, r22		; Add without carry r22 to r24, placing the result in r24
	brcs basino_add_had_carry
	ldi r25, 0
	rjmp basino_add_end

basino_add_had_carry:
	ldi r25, 1

basino_add_end:
	ret


;; Address add function.  Can be used to initialize variables or for
;; other things.
;; Adds two 16-bit values, returns an error if there is a carry.
;; Parameters:
;; parameter1: 16-bit first add operand
;; parameter2: 16-bit second add operand
;; parameter3: 16-bit reference to return result
;; return:     16-bit result of addition
;; Need to add:    r25 r24
;;               + r23 r22
;; So, add r24 and r22 first (add), then r25 + r23 with carry (adc)
basino_address_add:
	push r26
	push r27
	push r28
	push r29

	add r24, r22		; Add without carry the low bits, placing the result in r24
	adc r25, r23		; Add with carry the high bits, placing the result in r25

	brcs basino_address_add_had_carry
	ldi r30, 0
	rjmp basino_address_add_end

basino_address_add_had_carry:
	ldi r30, 1

basino_address_add_end:
	;; Copy argument three (the return result reference)
	movw X, r20
	st X, r30

	pop r29
	pop r28
	pop r27
	pop r26

	ret


;; The Sleep Mode Control Register
;; Contains the sleep enable bit, and the sleep mode bits
;; Some references:
;; AVR-InstructionSet-Manual-DS40002198.pdf: Page 10
;; This isn't memory mapped (0x55), see for example m644Pdef.inc for
;; the ATmega644P
.set SMCR, 0x33		        ; The Sleep Mode Control Register
.set SE, 0			; The sleep enable bit
.set SM0, 1			; Sleep Mode 0
.set SM1, 2
.set SM2, 3

;; To enter power-down mode:
;; First, set the sleep enable bit and mode in SMCR.
;; Then execute a sleep instruction.
;; Interrupts can be enabled to manage waking from sleep mode,
;; but that's not done here.
power_down:
	;; This mask enables sleep, and sets the sleep mode to power-down
	ldi r16, (1<<SE) | (0<<SM0) | (1<<SM1) | (0<< SM2)
 	out SMCR, r16
	sleep
	;; The ret doesn't get executed until we receive an interrupt
	;; or other event to wake the device
	ret
