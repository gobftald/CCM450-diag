ENTRY(_start)

PROVIDE(_stext = ORIGIN(ROTEXT));
PROVIDE(_max_hart_id = 0);

PROVIDE(ExceptionHandler = DefaultExceptionHandler);

PROVIDE(__post_init = default_post_init);

/* esp32c3 fixups */

SECTIONS {
  .trap : ALIGN(4)
  {
    KEEP(*(.trap.vector));  /* if this is the first one, we can save space due to padding */
    KEEP(*(.trap));
    *(.trap.*);
    . = ALIGN(4);
  } > RWTEXT
}
INSERT BEFORE .rwtext;

SECTIONS {
    .rotext_dummy (NOLOAD) :
    {
        /* This dummy section represents the .rodata section within ROTEXT.
        * Since the same physical memory is mapped to both DROM and IROM,
        * we need to make sure the .rodata and .text sections don't overlap.
        * We skip the amount of memory taken by .rodata* in .text
        */

        /* Start at the same alignment constraint than .flash.text */
        
        . = ALIGN(ALIGNOF(.rodata));

        /* Create an empty gap as big as .text section */

        . = . + SIZEOF(.rodata) + SIZEOF(.rodata.wifi);
        
        /* Prepare the alignment of the section above. Few bytes (0x20) must be
        * added for the mapping header.
        */

        . = ALIGN(0x10000) + 0x20;
    
    } > ROTEXT
}
#INSERT BEFORE .text;
# if you insert debug.x in text.x
INSERT BEFORE .esp-riscv-rt;

/* Similar to .rotext_dummy this represents .rwtext but in .data */
SECTIONS {
    .rwdata_dummy (NOLOAD) : ALIGN(4)
    {
        . = . + SIZEOF(.trap) + SIZEOF(.rwtext) + SIZEOF(.rwtext.wifi);

    }  > RWDATA
}
INSERT BEFORE .data;

/* Must be called __global_pointer$ for linker relaxations to work. */
PROVIDE(__global_pointer$ = _data_start + 0x800);

/* end of esp32c3 fixups */

/* Shared sections - ordering matters */
SECTIONS {
    INCLUDE "rwtext.x"
    INCLUDE "rwdata.x"
}

INCLUDE "rodata.x"
INCLUDE "text.x"
INCLUDE "stack.x"
/* End of Shared sections */

INCLUDE "debug.x"

_dram_origin = ORIGIN( DRAM );
