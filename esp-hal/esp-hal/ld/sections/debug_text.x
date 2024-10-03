    .esp-riscv-rt : ALIGN(4)
    {
        *(.init)
        *(.init.rust)
        *(.text.abort)
        *(.text.DefaultExceptionHandler)
        *(.text.default_post_init)
    } > ROTEXT

    .hal : ALIGN(4)
    {
        *(.text._setup_interrupts)
        *(.text.EspDefaultHandler)
        *(.text.hal_main)
        *(.text.*esp_hal*embassy*)
        *(.text.__pender)
        *(.text.*esp_hal*)
    } > ROTEXT

    .esp-println : ALIGN(4)
    {
        *(.text._critical_section_1_0_acquire)
        *(.text._critical_section_1_0_release)
        *(.text.*esp_println*)
    } > ROTEXT

    .embassy-exec : ALIGN(4)
    {
        *(.text.*embassy_executor*)
    } > ROTEXT

    .core : ALIGN(4)
    {
        *(.text.*core*)
    } > ROTEXT
