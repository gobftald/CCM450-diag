.esp_rom_sys : ALIGN(4)
{
    *(.text.unlikely.__assert_func)
    
    *(.text.__atoi)
    *(.text.__mktime)
    *(.text.__strnlen)
    
} > ROTEXT