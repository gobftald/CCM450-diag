.esp-alloc : ALIGN(4)
{
    *(.text.*esp_alloc*malloc*malloc_with_caps*)
    *(.text.*esp_alloc*malloc*realloc_with_caps*)

    *(.text.calloc_internal)
    *(.text.free)
    *(.text.get_free_internal_heap_size)
    *(.text.malloc)
    *(.text.malloc_internal)
    *(.text.realloc_internal)

} > ROTEXT
