SECTIONS {
  INCLUDE "debug_text.x"  # adjust 'INSERT BEFORE' line after '.text.dummy' SECTION in esp32c3.x,
                          # replacing '.text' to the first SECTION name in debug_text.x

  .text : ALIGN(4)
  {
    #IF riscv
    KEEP(*(.init));
    KEEP(*(.init.rust));
    KEEP(*(.text.abort));
    #ENDIF
    *(.literal .text .literal.* .text.*)
    _etext = ABSOLUTE(.);
  } > ROTEXT

}
