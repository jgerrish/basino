/*
 * Default settings for a avr-gcc compiled static AVR library:
 * .bss depends on the size and start of .data
 * Name     Addr
 * .data    00800060
 * .text    00000000
 * .bss     008000?? (starts after the .data section)
 *
 * Defaults for a Rust compiled AVR application linked with a static C library
 * .bss depends on the size and start of .data
 * .data    00800100
 * .text    00000000
 * .bss     00800??? (starts after the .data section)
 *
 * Nim also puts the data section at 0x00800100
 */

/* This uses the same layout as the default AVR linker script */
/* The value of 0x00800000 is a prefix to indicate that the .data should go in data
 * memory */

/* 0x0100 is the correct location, 0x0060 is the start of extended I/O registers
 * See DS40002061B-page 19 */
MEMORY
{
  /* 32256, because the flash memory size is 32 KB - 512 bytes for the bootloader */
  /* This might be a wrong value */
  PROGMEM (rwa) : ORIGIN = 0x00000000, LENGTH = 32256
  SRAM (rwa) : ORIGIN = 0x00800100, LENGTH = 768
  XSRAM (rwa) : ORIGIN = 0x00800400, LENGTH = 1K
}

SECTIONS
{
  . = 0x00000000;
  .text : { *(.text) }  > PROGMEM
  . = 0x00800100;
  .data : { *(.data) } > SRAM
  .bss : { *(.bss) } > SRAM
  /* TODO: Add some sections here to put our data in */
  /* The Atmega 328p has 2KB of SRAM.  Start this at 0x300 (512 bytes of memory). */
  /* This will need to be adjusted.
  /* . = 0x00800300; */
  .ram2bss : { *(.ram2bss) } > ESRAM
  /* . = 0x00800300; */
  .hisram : { *(.hisram) } > ESRAM
}
