
/* original content
SECTIONS {

  .text : ALIGN(4)
  {
    #IF riscv
    KEEP(*(.init));
    KEEP(*(.init.rust));
    KEEP(*(.text.abort));
    #ENDIF
    *(.literal .text .literal.* .text.*)
  } > ROTEXT

}
*/

.text_etc : ALIGN(4)
{
/*
    #IF riscv
    KEEP(*(.init));
    KEEP(*(.init.rust));
    KEEP(*(.text.abort));
    #ENDIF
    */

    /* second part of the spitted original
        the first part is in debug_init.x */
    *(.literal .text .literal.* .text.*)

} > ROTEXT
