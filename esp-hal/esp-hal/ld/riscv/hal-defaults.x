PROVIDE(_pre_init_trap = _default_abort);

    EXTERN(_default_start_trap);
PROVIDE(_start_trap = _default_start_trap);

PROVIDE(DefaultHandler = EspDefaultHandler);

/* don't init data - expect the bootloader to do it */
__sdata = 0;
__edata = 0;
__sidata = 0;

/* alias bss start + end as expected by riscv-rt */
__sbss = _bss_start;
__ebss = _bss_end;

PROVIDE(interrupt1 = DefaultHandler);
PROVIDE(interrupt2 = DefaultHandler);
PROVIDE(interrupt3 = DefaultHandler);
PROVIDE(interrupt4 = DefaultHandler);
PROVIDE(interrupt5 = DefaultHandler);
PROVIDE(interrupt6 = DefaultHandler);
PROVIDE(interrupt7 = DefaultHandler);
PROVIDE(interrupt8 = DefaultHandler);
PROVIDE(interrupt9 = DefaultHandler);
PROVIDE(interrupt10 = DefaultHandler);
PROVIDE(interrupt11 = DefaultHandler);
PROVIDE(interrupt12 = DefaultHandler);
PROVIDE(interrupt13 = DefaultHandler);
PROVIDE(interrupt14 = DefaultHandler);
PROVIDE(interrupt15 = DefaultHandler);
PROVIDE(interrupt16 = DefaultHandler);
PROVIDE(interrupt17 = DefaultHandler);
PROVIDE(interrupt18 = DefaultHandler);
PROVIDE(interrupt19 = DefaultHandler);
PROVIDE(interrupt20 = DefaultHandler);
PROVIDE(interrupt21 = DefaultHandler);
PROVIDE(interrupt22 = DefaultHandler);
PROVIDE(interrupt23 = DefaultHandler);
PROVIDE(interrupt24 = DefaultHandler);
PROVIDE(interrupt25 = DefaultHandler);
PROVIDE(interrupt26 = DefaultHandler);
PROVIDE(interrupt27 = DefaultHandler);
PROVIDE(interrupt28 = DefaultHandler);
PROVIDE(interrupt29 = DefaultHandler);
PROVIDE(interrupt30 = DefaultHandler);
PROVIDE(interrupt31 = DefaultHandler);

INCLUDE "device.x"

EXTERN(_default_abort);
PROVIDE(abort = _default_abort);
PROVIDE(ExceptionHandler = abort);