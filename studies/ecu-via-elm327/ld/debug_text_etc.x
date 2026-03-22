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

.text : ALIGN(4)
{
    /* first part of the spitted original
        the second part is in debug_text.x */
    /*
    #IF riscv
    KEEP(*(.init));
    KEEP(*(.init.rust));
    KEEP(*(.text.abort));
    #ENDIF
    */

    *(.literal .text .literal.* .text.*)

} > ROTEXT
