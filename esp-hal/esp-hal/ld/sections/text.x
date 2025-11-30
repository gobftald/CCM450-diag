SECTIONS {
    INCLUDE "debug_text.x"      # adjust 'INSERT BEFORE' line after '.rotext_dummy' SECTION in esp32c3.x,
                            	# replacing '.text' to the first SECTION name in text_debug.x

    .text : ALIGN(4)
    {

        #IF riscv
            KEEP(*(.init));
            KEEP(*(.init.rust));
            KEEP(*(.text.abort));
        #ENDIF

        /* application */
        *(.text.*embassy_executor*raw*TaskStorage*poll*)
        *(.text.*ecu_via_elm327*adapter*)
        *(.text.*___rustc35___rust_no_alloc_shim_is_unstable_v2)
        *(.text.main)

        *(.literal .text .literal.* .text.*)
        _etext = ABSOLUTE(.);

    } > ROTEXT
}