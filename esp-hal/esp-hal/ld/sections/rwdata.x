.data : ALIGN(4)
{
    _data_start = ABSOLUTE(.);
    . = ALIGN (4);

    *(.sdata .sdata.* .sdata2 .sdata2.*);
    *(.data .data.*);
    *(.data1)

    _data_end = ABSOLUTE(.);
    . = ALIGN(4);

} > RWDATA

.bss (NOLOAD) : ALIGN(4)
{
    _bss_start = ABSOLUTE(.);
    . = ALIGN (4);

    *(.sbss)
    *(.sbss.*)
    *(.scommon)
    *(.sbss2)
    *(.sbss2.*)
    *(.sbss .sbss.* .bss .bss.*);

    _bss_end = ABSOLUTE(.);
    . = ALIGN(4);

} > RWDATA
