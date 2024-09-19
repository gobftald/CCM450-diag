ENTRY(_start)

PROVIDE(_stext = ORIGIN(ROTEXT));
PROVIDE(_max_hart_id = 0);

# if ExceptionHandler is not defined in e.g. esp-backtrace
PROVIDE(ExceptionHandler = DefaultExceptionHandler);

PROVIDE(__post_init = default_post_init);

/* esp32c3 fixups */

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
    . = . + SIZEOF(.text);
    /* Prepare the alignement of the section above. Few bytes (0x20) must be
     * added for the mapping header. */
    . = ALIGN(0x10000) + 0x20;
  } > RODATA
}
INSERT BEFORE .rodata;

/* Shared sections - ordering matters */
INCLUDE "text.x"
INCLUDE "rodata.x"
INCLUDE "rwtext.x"
INCLUDE "rwdata.x"
INCLUDE "stack.x"
/* End of Shared sections */

INCLUDE "debug.x"