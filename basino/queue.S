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


;; These are defines that are used to calculate offsets into the queue
;; data structure.
;;
;; Putting an item in the queue requires a bunch of comparisons
;; to find if the queue is full.  There are several ways this can
;; be optimized.
;; If the queue size is a power of two, we can mask and
;; test for zero, this helps with end-of-queue calculations.
;; That isn't done in this codebase yet.
;; One thing we do is keep track of the last position of the head.
;; Then we can do a simple load and equality comparison, instead of
;; load, increment, bounds check and equality.
;;
;; last_head is compared against the tail when inserting an item into
;; the queue.
;; It is not used for other checks.
.set BASINO_QUEUE_QUEUE_DATA_OFFSET, 0
.set BASINO_QUEUE_QUEUE_START_OFFSET, 2
.set BASINO_QUEUE_QUEUE_END_OFFSET, 4
.set BASINO_QUEUE_QUEUE_HEAD_OFFSET, 6
.set BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET, 8
.set BASINO_QUEUE_QUEUE_TAIL_OFFSET, 10

;; Initialize the queue
;;
;; Parameters:
;;   parameter 1: A pointer to the queue structure.  It is placed in r24 and r25
;;   parameter 2: A pointer to the start of the queue
;;   parameter 3: A pointer to the end of the queue
;;
;; Returns: A result code: 0 indicates success, 1 indicates failure
;;
basino_queue_init:
	push r16
	push r22
	push r23
	push r25
	push r26
	push r27
	push r28
	push r29

basino_queue_init_null_pointer_check:
	;; Check that the first parameter is not a null pointer
	ldi r16, 0
	cp r24, r16
	brne basino_queue_init_do_init
	cp r25, r16
	brne basino_queue_init_do_init
	rjmp basino_queue_init_null_pointer

basino_queue_init_do_init:
	movw Y, r24		; Copy r25:r24 -> Y

	;; Set the pointers in the queue structure to actual locations
	;; in the queue.  This assumes a normal array that is
	;; continuous.
	std Y+BASINO_QUEUE_QUEUE_START_OFFSET, r22
	std Y+BASINO_QUEUE_QUEUE_START_OFFSET+1, r23

	std Y+BASINO_QUEUE_QUEUE_END_OFFSET, r20
	std Y+BASINO_QUEUE_QUEUE_END_OFFSET+1, r21

	std Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET, r22
	std Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET+1, r23

	std Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET, r20
	std Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET+1, r21

	std Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET, r22
	std Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET+1, r23

	rjmp basino_queue_init_successful

basino_queue_init_null_pointer:
	;; Set the return result to NULL pointer error
	ldi r24, 0x01
	rjmp basino_queue_init_end

basino_queue_init_successful:
	;; Set the return result to successful
	ldi r24, 0x00
	rjmp basino_queue_init_end

basino_queue_init_end:
	pop r29
	pop r28
	pop r27
	pop r26
	pop r25
	pop r23
	pop r22
	pop r16

	ret

;; Add an item to the queue
;;
;; Parameters:
;;   parameter 1: The pointer to the queue structure (r24 and r25)
;;   parameter 2: The 8-bit unsigned integer to add (r22 and r23)
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
	push r16
	push r17
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

basino_queue_put_null_pointer_check:
	;; Test for a null pointer as the first parameter
	ldi r16, 0
	cp r24, r16
	brne basino_queue_put_not_null_pointer
	cp r25, r16
	brne basino_queue_put_not_null_pointer
	rjmp basino_queue_put_null_pointer

basino_queue_put_not_null_pointer:
	;; The first parameter is a pointer to the queue structure.
	;; This structure contains the start, end, head, last_head and tail
	;; These are all pointers to locations in the queue data buffer.
	;;
	;; This copies r25:r24, the first parameter to the Y-Pointer Register
	movw Y, r24		; Copy r25:r24 -> Y

	;; This uses offsets into the structure to load registers with
	;; the pointers.
	;; For examaple, the two instructions below load the head into r19:r18
	;; After the instruction executes, r19:r18 will contain an address pointing
	;; to the head of the queue in the data buffer.
	ldd r18, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET
	ldd r19, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET+1

	;; This loads r27:r26 (the X-Pointer Register) with the tail.
	;; We can then use the X Pointer Register to get the contents
	;; of the queue.
	ldd r26, Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET
	ldd r27, Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET+1

	movw r14, X		; Copy X to r15:r14

	movw r20, X		; Copy X to r21:r20

	;; Check for case when queue is full
	rjmp basino_queue_put_check_full

basino_queue_put_check_full:
	;; If adding an item would make the queue head equal the queue
	;; tail, the queue is full.
	;; We have an extra variable, basino_queue_last_head to make
	;; this check easier.
	ldd r16, Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET
	ldd r17, Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET+1

	;; Check the lower byte
	cp r16, r20

	;; Check the upper byte
	breq basino_queue_put_low_byte_equal
	;; not equal, so we're not full
	rjmp basino_queue_put_do_put

basino_queue_put_low_byte_equal:
	;; If the lower byte of the last head and the tail is equal,
	;; check the upper byte.
	cp r17, r21
	breq basino_queue_put_full
	rjmp basino_queue_put_do_put

basino_queue_put_do_put:
	movw X, r20		; Copy r21:r20 (current value of basino_queue_tail) to X
	st X+, r22		; Store the item in the queue and advance

	;; Store the tail of the queue, handling the case where it's at the
	;; end of the queue.
	ldd r18, Y+BASINO_QUEUE_QUEUE_END_OFFSET
	ldd r19, Y+BASINO_QUEUE_QUEUE_END_OFFSET+1

	;; Can also use cpse here and rjmp instead of brne basino_queue_put_store_tail_real
	cp r18, r20		; compare low byte of the tail to the end of the queue
	brne basino_queue_put_store_tail
	cp r19, r21		; compare high byte of the tail to the end of the queue
	brne basino_queue_put_store_tail

	;; Deal with the case where we're at the end of the queue
	;; Wrap to the beginning
	movw X, r14		; Copy saved basino_queue_tail address to X

	ldd r20, Y+BASINO_QUEUE_QUEUE_START_OFFSET
	ldd r21, Y+BASINO_QUEUE_QUEUE_START_OFFSET+1

	std Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET, r20
	std Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET+1, r21

	rjmp basino_queue_put_success

basino_queue_put_store_tail:
	;; Store the the new queue tail
	std Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET, r26
	std Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET+1, r27

	rjmp basino_queue_put_success

basino_queue_put_full:
	ldi r24, 0x02
	rjmp basino_queue_put_end

basino_queue_put_null_pointer:
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
	pop r17
	pop r16
	pop r15
	pop r14

	ret

;; Get an item from the queue
;;
;; Parameters:
;;   parameter 1: The pointer to the queue structure
;;   parameter 2: A 16-bit pointer to an 8-bit return result
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

basino_queue_get_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_queue_get_not_null_pointer
	cp r25, r16
	brne basino_queue_get_not_null_pointer
	rjmp basino_queue_get_null_pointer

basino_queue_get_not_null_pointer:
	movw Y, r24		; Copy r25:r24 -> Y

	;; Check for case when head == tail
	rjmp basino_queue_get_check_empty

basino_queue_get_check_empty:
	ldd r26, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET
	ldd r27, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET+1

	movw r16, X		; This copies X to r17:r16

	movw r18, X		; This copies X to r19:r18

	ldd r20, Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET
	ldd r21, Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET+1

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
	ldi r30, 0x02
	rjmp basino_queue_get_end

basino_queue_get_do_get:
	;; First, backup the current head to make put operations
	;; easier.
	;; Copy r19:r18 (the value of basino_queue_head) to X (r27:r26)
	movw X, r18

	ldd r26, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET
	ldd r27, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET+1

	ldd r20, Y+BASINO_QUEUE_QUEUE_END_OFFSET
	ldd r21, Y+BASINO_QUEUE_QUEUE_END_OFFSET+1

	;; We do a comparison here to see if we're at the end of the
	;; queue before we get the value.
	cp r26, r20		; compare low byte of the head to the end of the queue
	brne basino_queue_get_eoq_test_false
	cp r27, r21		; compare high byte of the head to the end of the queue
	brne basino_queue_get_eoq_test_false

	;; We're at the end of the queue, we have to wrap the head to
	;; the beginning

	;; Deal with the case where we're at the end of the queue
	;; Wrap to the beginning
	movw X, r16		; Copy saved basino_queue_head address to X
	ld r14, X+		; Get the item in the queue and advance

	ldd r20, Y+BASINO_QUEUE_QUEUE_START_OFFSET
	ldd r21, Y+BASINO_QUEUE_QUEUE_START_OFFSET+1

	std Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET, r20
	std Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET+1, r21

	std Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET, r16
	std Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET+1, r17

	rjmp basino_queue_get_success

basino_queue_get_eoq_test_false:
	;; We're not at the end of the queue, so we don't have to
	;; worry about wrapping the head

	;; Get the current value and advance the pointer
	movw X, r16		; Copy saved basino_queue_head address to X
	ld r14, X+		; Get the item in the queue and advance

	rjmp basino_queue_get_store_head

basino_queue_get_store_head:
	;; This is setting the item the head points to to the head
	;; We want to change the head pointer itself
	std Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET, r26
	std Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET+1, r27

	;; Update last head
	std Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET, r16
	std Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET+1, r17

	rjmp basino_queue_get_success

basino_queue_get_null_pointer:
	ldi r30, 0x01
	rjmp basino_queue_get_end

basino_queue_get_success:
	ldi r30, 0x00
	rjmp basino_queue_get_end

basino_queue_get_end:
	movw X, r22		; Copy r23:r22 (second parameter) to X
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
;;
;; Parameters:
;;   parameter 1: The pointer to the queue structure
;;   parameter 2: A pointer to an 8-bit return result
;;                On success the value is zero,
;;                on failure the value is non-zero.
;;
;; Returns: A pointer to the last head of the queue.
basino_queue_get_last_head:
	push r16
	push r28
	push r29
	push r30

basino_queue_get_last_head_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_queue_get_last_head_not_null_pointer
	cp r25, r16
	brne basino_queue_get_last_head_not_null_pointer
	rjmp basino_queue_get_last_head_null_pointer

basino_queue_get_last_head_not_null_pointer:
	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET
	ldd r25, Y+BASINO_QUEUE_QUEUE_LAST_HEAD_OFFSET+1

	rjmp basino_queue_get_last_head_success

basino_queue_get_last_head_success:
	ldi r30, 0x00

	rjmp basino_queue_get_last_head_end

basino_queue_get_last_head_null_pointer:
	ldi r30, 0x01
	rjmp basino_queue_get_last_head_end

basino_queue_get_last_head_end:
	movw X, r22		; Copy r23:r22 (second parameter) to X
	st X, r30

	pop r30
	pop r29
	pop r28
	pop r16

	ret

;; Get the current head pointer
;; The head is updated on a get
;;
;; Parameters:
;;   parameter 1: The pointer to the queue structure
;;   parameter 2: A pointer to an 8-bit return result
;;                On success the value is zero,
;;                on failure the value is non-zero.
;;
;; Returns: A pointer to the head of the queue.
basino_queue_get_head:
	push r16
	push r28
	push r29
	push r30

basino_queue_get_head_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_queue_get_head_not_null_pointer
	cp r25, r16
	brne basino_queue_get_head_not_null_pointer
	rjmp basino_queue_get_head_null_pointer

basino_queue_get_head_not_null_pointer:
	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET
	ldd r25, Y+BASINO_QUEUE_QUEUE_HEAD_OFFSET+1

	rjmp basino_queue_get_head_success

basino_queue_get_head_success:
	ldi r30, 0x00

	rjmp basino_queue_get_head_end

basino_queue_get_head_null_pointer:
	ldi r30, 0x01
	rjmp basino_queue_get_head_end

basino_queue_get_head_end:
	movw X, r22		; Copy r23:r22 (second parameter) to X
	st X, r30

	pop r30
	pop r29
	pop r28
	pop r16

	ret

;; Get the current tail pointer
;; The tail is updated on a put
;;
;; Parameters:
;;   parameter 1: The pointer to the queue structure
;;   parameter 2: A pointer to an 8-bit return result
;;                On success the value is zero,
;;                on failure the value is non-zero.
;;
;; Returns: A pointer to the tail of the queue.
basino_queue_get_tail:
	push r16
	push r28
	push r29
	push r30

basino_queue_get_tail_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_queue_get_tail_not_null_pointer
	cp r25, r16
	brne basino_queue_get_tail_not_null_pointer
	rjmp basino_queue_get_tail_null_pointer

basino_queue_get_tail_not_null_pointer:
	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET
	ldd r25, Y+BASINO_QUEUE_QUEUE_TAIL_OFFSET+1

	rjmp basino_queue_get_tail_success

basino_queue_get_tail_success:
	ldi r30, 0x00

	rjmp basino_queue_get_tail_end

basino_queue_get_tail_null_pointer:
	ldi r30, 0x01
	rjmp basino_queue_get_tail_end

basino_queue_get_tail_end:
	movw X, r22		; Copy r23:r22 (second parameter) to X
	st X, r30

	pop r30
	pop r29
	pop r28
	pop r16

	ret

;; Get a pointer to the start of the queue
;;
;; Parameters:
;;   parameter 1: The pointer to the queue structure
;;   parameter 2: A pointer to an 8-bit return result
;;                On success the value is zero,
;;                on failure the value is non-zero.
;;
;; Returns: A pointer to the start of the queue.
basino_queue_get_queue_start:
	push r16
	push r28
	push r29
	push r30

basino_queue_get_queue_start_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_queue_get_queue_start_not_null_pointer
	cp r25, r16
	brne basino_queue_get_queue_start_not_null_pointer
	rjmp basino_queue_get_queue_start_null_pointer

basino_queue_get_queue_start_not_null_pointer:
	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_QUEUE_QUEUE_START_OFFSET
	ldd r25, Y+BASINO_QUEUE_QUEUE_START_OFFSET+1

	rjmp basino_queue_get_queue_start_success

basino_queue_get_queue_start_success:
	ldi r30, 0x00

	rjmp basino_queue_get_queue_start_end

basino_queue_get_queue_start_null_pointer:
	ldi r30, 0x01
	rjmp basino_queue_get_queue_start_end

basino_queue_get_queue_start_end:
	movw X, r22		; Copy r23:r22 (second parameter) to X
	st X, r30

	pop r30
	pop r29
	pop r28
	pop r16

	ret

;; Get a pointer to the end of the queue
;;
;; Parameters:
;;   parameter 1: The pointer to the queue structure
;;   parameter 2: A pointer to an 8-bit return result
;;                On success the value is zero,
;;                on failure the value is non-zero.
;;
;; Returns: A pointer to the end of the queue.
basino_queue_get_queue_end:
	push r16
	push r28
	push r29
	push r30

basino_queue_get_queue_end_null_pointer_check:
	ldi r16, 0
	cp r24, r16
	brne basino_queue_get_queue_end_not_null_pointer
	cp r25, r16
	brne basino_queue_get_queue_end_not_null_pointer
	rjmp basino_queue_get_queue_end_null_pointer

basino_queue_get_queue_end_not_null_pointer:
	movw Y, r24		; Copy r25:r24 -> Y

	ldd r24, Y+BASINO_QUEUE_QUEUE_END_OFFSET
	ldd r25, Y+BASINO_QUEUE_QUEUE_END_OFFSET+1

	rjmp basino_queue_get_queue_end_success

basino_queue_get_queue_end_success:
	ldi r30, 0x00

	rjmp basino_queue_get_queue_end_end

basino_queue_get_queue_end_null_pointer:
	ldi r30, 0x01
	rjmp basino_queue_get_queue_end_end

basino_queue_get_queue_end_end:
	movw X, r22		; Copy r23:r22 (second parameter) to X
	st X, r30

	pop r30
	pop r29
	pop r28
	pop r16

	ret
