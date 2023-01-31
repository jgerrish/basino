;; AVR queue / buffer implementation.
;;
;; A simple implementation of a queue or buffer on AVR devices.
;;
;; A queue is a First-In-First-Out (FIFO) data structure.
;; This queue is backed by an array or other continuous region of
;; memory,  A pointer to this region is passed into the init function.
;;
;; There are two operations defined on the queue:
;;   put: put an item into the queue
;;   get: get an item from the queue
;;
;; The queue has a head and tail pointer.  The head points to the
;; location where the next item should be placed.  The tail points
;; to the location where the next item should be retrieved on get.
;;
;; These tail and head pointers wrap around when they reach the end of
;; the array.
;;
;; Here's the queue after init, both head and tail point to the the
;; first location.
;;
;;   +-------------------------------------------+
;;   |   | ................................. |   |
;;   +-------------------------------------------+
;;    ^ ^                                      ^
;;    | |                                      |
;; head tail                                 last
;;                                           head
;;
;;
;; Here's a diagram of a queue with the ASCII letters Hello in it:
;;
;; Nothing has been read from the queue, it has had five put
;; operations:
;;
;;   +-------------------------------------------+
;;   | H | e | l | l | o |   | ............. |   |
;;   +-------------------------------------------+
;;     ^                   ^                   ^
;;     |                   |                   |
;;   head                tail                last
;;                                           head
;;
;; After a get, it returns 'H' and looks like this:
;;
;;   +-------------------------------------------+
;;   | H | e | l | l | o |   | ............. |   |
;;   +-------------------------------------------+
;;     ^   ^               ^
;;     |   |               |
;;   last  head           tail
;;   head
;;

.text

.globl basino_queue_init, basino_queue_put, basino_queue_get


;; Initialize the queue
;;
;; Parameters:
;;   parameter 1: A pointer to the start of the queue
;;   parameter 2: A pointer to the end of the queue
;;
;; Returns: A result code: 0 indicates success, 1 indicates failure
;;
basino_queue_init:
	push r22
	push r23
	push r25
	push r26
	push r27

	ldi XL, lo8(basino_queue_start)
	ldi XH, hi8(basino_queue_start)

	st X+, r24
	st X, r25

	ldi XL, lo8(basino_queue_head)
	ldi XH, hi8(basino_queue_head)

	st X+, r24
	st X, r25

	ldi XL, lo8(basino_queue_tail)
	ldi XH, hi8(basino_queue_tail)

	st X+, r24
	st X, r25

	;; Set the last head position to the end of the queue
	ldi XL, lo8(basino_queue_last_head)
	ldi XH, hi8(basino_queue_last_head)

	st X+, r22
	st X, r23

	;; Set the basino_stack_bottom variable to the maximum bottom of the stack
	ldi XL, lo8(basino_queue_end)
	ldi XH, hi8(basino_queue_end)

	st X+, r22
	st X, r23

	;; Set the return result to successful
	ldi r24, 0x00

	pop r27
	pop r26
	pop r25
	pop r23
	pop r22

	ret

;; Add an item to the queue
;;
;; Parameters:
;;   parameter 1: The 8-bit unsigned integer to add
;;
;; Returns: A result code: 0 indicates success, 1 indicates failure
;;
;; Consistent usage of registers throughout the subroutine:
;; r15:r14 contains basino_queue_tail (the address, not the contents)
;; r17:r16 contains basino_queue_head (the address, not the contents)
;;
basino_queue_put:
	push r14
	push r15
	push r18
	push r19
	push r20
	push r21
	push r22
	push r23
	push r26
	push r27
	push r28
	push r29

	ldi XL, lo8(basino_queue_head)
	ldi XH, hi8(basino_queue_head)

	ld r18, X+
	ld r19, X

	ldi XL, lo8(basino_queue_tail)
	ldi XH, hi8(basino_queue_tail)

	movw r14, X		; Copy X to r15:r14

	ld r20, X+
	ld r21, X

	;; Check for case when queue is full
	rjmp basino_queue_put_check_full

basino_queue_put_check_full:
	;; If adding an item would make the queue head equal the queue
	;; tail, the queue is full.
	;; We have an extra variable, basino_queue_last_head to make
	;; this check easier.
	ldi YL, lo8(basino_queue_last_head)
	ldi YH, hi8(basino_queue_last_head)
	ld r22, Y+
	ld r23, Y

	;; Check the lower byte
	cp r22, r20

	;; Check the upper byte
	breq basino_queue_put_low_byte_equal
	;; not equal, so we're not full
	rjmp basino_queue_put_do_put

basino_queue_put_low_byte_equal:
	;; If the lower byte of the last head and the tail is equal,
	;; check the upper byte.
	cp r23, r21
	breq basino_queue_put_full
	rjmp basino_queue_put_do_put

basino_queue_put_do_put:
	movw X, r20		; Copy r21:r20 (current value of basino_queue_tail) to X
	st X+, r24		; Store the item in the queue and advance

	;; Store the end of the queue, handling the case where it's at the
	;; end of the queue.
	ldi YL, lo8(basino_queue_end)
	ldi YH, hi8(basino_queue_end)
	ld r18, Y+
	ld r19, Y

	;; Can also use cpse here and rjmp instead of brne basino_queue_put_store_tail_real
	cp r18, r20		; compare low byte of the tail to the end of the queue
	brne basino_queue_put_store_tail
	cp r19, r21		; compare high byte of the tail to the end of the queue
	brne basino_queue_put_store_tail

	;; Deal with the case where we're at the end of the queue
	;; Wrap to the beginning
	movw X, r14		; Copy saved basino_queue_tail address to X

	ldi YL, lo8(basino_queue_start)
	ldi YH, hi8(basino_queue_start)
	ld r20, Y+
	ld r21, Y

	st X+, r20
	st X, r21

	rjmp basino_queue_put_success

basino_queue_put_store_tail:
	movw Y, r14		; This copies r15:r14 (basino_queue_tail) to Y
	st Y+, r26
	st Y, r27

	rjmp basino_queue_put_success

basino_queue_put_full:
	ldi r24, 0x01
	rjmp basino_queue_put_end

basino_queue_put_success:
	ldi r24, 0x00
	rjmp basino_queue_put_end

basino_queue_put_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r23
	pop r22
	pop r21
	pop r20
	pop r19
	pop r18
	pop r15
	pop r14

	ret

;; Get an item from the queue
;;
;; Parameters:
;;   parameter 1: A 16-bit pointer to an 8-bit return result
;;                On success the value is zero,
;;                on failure the value is non-zero.
;;
;; Returns: The value in the queue
;;
;; Consistent usage of registers throughout the subroutine:
;; r17:r16 contains basino_queue_head (the address, not the contents)
;;
basino_queue_get:
	push r14
	push r16
	push r17
	push r18
	push r19
	push r20
	push r21
	push r26
	push r27
	push r28
	push r29
	push r30

	;; Check for case when head == tail
	rjmp basino_queue_get_check_empty

basino_queue_get_check_empty:
	ldi XL, lo8(basino_queue_head)
	ldi XH, hi8(basino_queue_head)

	movw r16, X		; This copies X to r17:r16

	ld r18, X+
	ld r19, X

	ldi XL, lo8(basino_queue_tail)
	ldi XH, hi8(basino_queue_tail)

	ld r20, X+
	ld r21, X

	;; Test that the queue is not empty
	;; Order of comparisons can actually matter here.
	;; Comparing the low-order byte first lets us avoid an
	;; unnecessary compare most of the time.
	cp r18, r20
	breq basino_queue_get_low_byte_equal
	rjmp basino_queue_get_do_get

basino_queue_get_low_byte_equal:
	;; Compare the high-order byte now
	cp r19, r21
	breq basino_queue_get_empty
	rjmp basino_queue_get_do_get

basino_queue_get_empty:
	ldi r30, 0x01
	rjmp basino_queue_get_end

basino_queue_get_do_get:
	;; First, backup the current head to make put operations
	;; easier.
	;; Copy r19:r18 (the value of basino_queue_head) to X (r27:r26)
	movw X, r18
	ldi YL, lo8(basino_queue_last_head)
	ldi YH, hi8(basino_queue_last_head)
	st Y+, r26
	st Y, r27

	ldi YL, lo8(basino_queue_end)
	ldi YH, hi8(basino_queue_end)
	ld r20, Y+
	ld r21, Y

	;; We do a comparison here to see if we're at the end of the
	;; queue before we get the value.
	;; The flags are still set after the value is retrieved, then
	;; we condtionally branch this comparison.
	cp r26, r20		; compare low byte of the head to the end of the queue
	brne basino_queue_get_eoq_test_false
	;; The following compare isn't acted on immediately
	cp r27, r21		; compare high byte of the head to the end of the queue

basino_queue_get_eoq_test_false:
	;; Get the current value and advance the pointer
	ld r14, X+		; Get the item in the queue and advance
	movw r18, X		; Copy X (r27:r26) -> r19:r18

	brne basino_queue_get_store_head

	;; Deal with the case where we're at the end of the queue
	;; Wrap to the beginning
	movw X, r16		; Copy saved basino_queue_head address to X

	ldi YL, lo8(basino_queue_start)
	ldi YH, hi8(basino_queue_start)
	ld r20, Y+
	ld r21, Y

	st X+, r20
	st X, r21

	rjmp basino_queue_get_success

basino_queue_get_store_head:
	;; Store the current head
	movw X, r16		; Copy r17:r16 (saved basino_queue_head address) -> X (r27:r26)

	st X+, r18
	st X, r19

	rjmp basino_queue_get_success

basino_queue_get_success:
	ldi r30, 0x00
	rjmp basino_queue_get_end

basino_queue_get_end:
	movw X, r24		; Copy r25:r24 (first parameter) to X
	st X, r30
	mov r24, r14		; Copy over the item to r24 for return

	pop r30
	pop r29
	pop r28
	pop r27
	pop r26
	pop r21
	pop r20
	pop r19
	pop r18
	pop r17
	pop r16
	pop r14

	ret

.globl basino_queue_get_last_head, basino_queue_get_head, basino_queue_get_tail
.globl basino_queue_get_queue_start, basino_queue_get_queue_end

;; Get the last head pointer
;; The last head pointer is used to simplify put operations
basino_queue_get_last_head:
	push r26
	push r27

	ldi XL, lo8(basino_queue_last_head)
	ldi XH, hi8(basino_queue_last_head)

	ld r24, X+
	ld r25, X

	pop r27
	pop r26

	ret

;; Get the current head pointer
;; The head is updated on a get
basino_queue_get_head:
	push r26
	push r27

	ldi XL, lo8(basino_queue_head)
	ldi XH, hi8(basino_queue_head)

	ld r24, X+
	ld r25, X

	pop r27
	pop r26

	ret

;; Get the current head pointer
;; The head is updated on a put
basino_queue_get_tail:
	push r26
	push r27

	ldi XL, lo8(basino_queue_tail)
	ldi XH, hi8(basino_queue_tail)

	ld r24, X+
	ld r25, X

	pop r27
	pop r26

	ret

;; Get a pointer to the start of the queue
basino_queue_get_queue_start:
	push r26
	push r27

	ldi XL, lo8(basino_queue_start)
	ldi XH, hi8(basino_queue_start)

	ld r24, X+
	ld r25, X

	pop r27
	pop r26

	ret

;; Get a pointer to the end of the queue
basino_queue_get_queue_end:
	push r26
	push r27

	ldi XL, lo8(basino_queue_end)
	ldi XH, hi8(basino_queue_end)

	ld r24, X+
	ld r25, X

	pop r27
	pop r26

	ret

.data

basino_queue_start:
	.short 0

basino_queue_end:
	.short 0

basino_queue_head:
	.short 0

;; Putting an item in the queue requires a bunch of comparisons
;; to find if the queue is full.  There are several ways this can
;; be optimized.
;; If the queue size is a power of two, we can mask and
;; test for zero, this helps with end-of-queue calculations.
;; That isn't done in this codebase yet.
;; One thing we do is keep track of the last position of the head.
;; Then we can do a simple load and equality comparison, instead of
;; load, increment, bounds check and equality.
basino_queue_last_head:
	.short 0

basino_queue_tail:
	.short 0

