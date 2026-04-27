.esp-sync : ALIGN(4)
{
    *(.text.*esp_sync*GenericRawMutex*lock_non_reentrant*)
    *(.text.*esp_sync*GenericRawMutex*lock*)
    *(.text.*esp_sync*RawMutex*lock*)
    *(.text.*esp_sync*LockGuard*new_non_reentrant*)
    *(.text.*esp_sync*NonReentrantMutex*with*)
    
} > ROTEXT