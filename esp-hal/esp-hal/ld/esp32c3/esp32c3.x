ENTRY(_start)

PROVIDE(_stext = ORIGIN(ROTEXT));
PROVIDE(_max_hart_id = 0);

# if ExceptionHandler is not defined in e.g. esp-backtrace
PROVIDE(ExceptionHandler = DefaultExceptionHandler);

PROVIDE(__post_init = default_post_init);

/* esp32c3 fixups */
SECTIONS {
  .text.dummy (NOLOAD) :
  {
    /* This section is intended to make _stext address work */
    . = ABSOLUTE(_stext);
  } > ROTEXT
}
#INSERT BEFORE .text;
# if you insert debug_text.x in text.x
INSERT BEFORE .esp-riscv-rt;

SECTIONS {
  .trap : ALIGN(4)
  {
    KEEP(*(.trap));
    *(.trap.*);
  } > RWTEXT
}
INSERT BEFORE .rwtext;

SECTIONS {
  /**
   * This dummy section represents the .text section but in rodata.
   * Thus, it must have its alignement and (at least) its size.
   */
  .text_dummy (NOLOAD):
  {
    /* Start at the same alignement constraint than .text */
    . = ALIGN(4);
    /* Create an empty gap as big as .text section */
    #. = . + SIZEOF(.text);
    #  Calculation method was changed to take account of debug_text.x
    . = . + (_etext - _stext);
    /* Prepare the alignement of the section above. Few bytes (0x20) must be
     * added for the mapping header. */
    . = ALIGN(0x10000) + 0x20;
  } > RODATA
}
INSERT BEFORE .rodata;

SECTIONS {
  /* similar as text_dummy */
  .rwdata_dummy (NOLOAD) : {
    . = ALIGN(ALIGNOF(.rwtext));
    . = . + SIZEOF(.rwtext);
    . = . + SIZEOF(.rwtext.wifi);
    . = . + SIZEOF(.trap);
  } > RWDATA
}
INSERT BEFORE .data;

/* Must be called __global_pointer$ for linker relaxations to work. */
PROVIDE(__global_pointer$ = _data_start + 0x800);
/* end of esp32c3 fixups */

/* Shared sections - ordering matters */
INCLUDE "text.x"
INCLUDE "rodata.x"
INCLUDE "rwtext.x"
INCLUDE "rwdata.x"
INCLUDE "stack.x"
/* End of Shared sections */

INCLUDE "debug.x"