.libprintf : ALIGN(4)
{
    *(.text._out_buffer)
    *(.text._out_null)
    *(.text._out_rev)
    *(.text._ntoa_format)
    *(.text._ntoa_long)
    *(.text._ntoa_long_long)
    *(.text._etoa)
    *(.text._ftoa)
    *(.text._vsnprintf)
    *(.text.sprintf)

} > ROTEXT
