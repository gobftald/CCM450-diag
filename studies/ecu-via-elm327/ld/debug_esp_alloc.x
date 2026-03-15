.esp-alloc : ALIGN(4)
{
    *(.text.*esp_alloc*HeapRegion*new*)
    *(.text.*esp_alloc*malloc*malloc_with_caps*)
    *(.text.*esp_alloc*malloc*realloc_with_caps*)
    *(.text.*esp_alloc*EspHeap*add_region*)

    *(.text.calloc)
    *(.text.calloc_internal)
    *(.text.free)
    *(.text.get_free_internal_heap_size)
    *(.text.malloc)
    *(.text.malloc_internal)
    *(.text.realloc_internal)

} > ROTEXT
