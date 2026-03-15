.ll-allocator : ALIGN(4)
{
    *(.text.*linked_list_allocator*Heap*deallocate*)
    *(.text.*linked_list_allocator*Heap*allocate_first_fit*)
    *(.text.*linked_list_allocator*Heap*top*)
    *(.text.*linked_list_allocator*Heap*free*)
    *(.text.*linked_list_allocator*Heap*init*)
    *(.text.*linked_list_allocator*Heap*empty*)

    *(.text.*linked_list_allocator*hole*check_merge_top*)
    *(.text.*linked_list_allocator*hole*HoleList*deallocate*)
    *(.text.*linked_list_allocator*hole*HoleList*align_layout*)
    *(.text.*linked_list_allocator*hole*HoleList*allocate_first_fit*)
    *(.text.*linked_list_allocator*hole*HoleList*new*)

    *(.text.*linked_list_allocator*align_up*)

} > ROTEXT