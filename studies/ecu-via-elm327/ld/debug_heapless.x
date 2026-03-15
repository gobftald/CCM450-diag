.heapless : ALIGN(4)
{
    *(.text.*heapless*linear_map*LinearMap*insert*)
    *(.text.*heapless*linear_map*LinearMap*remove*)

    *(.text.*heapless*vec*Vec*as*core*clone*Clone*clone*)
    *(.text.*heapless*vec*Vec*as*core*ops*drop*Drop*)
    *(.text.*heapless*vec*Vec*as*core*ops*deref*Deref*deref*)

    *(.text.*heapless*vec*Vec*swap_remove*)
    *(.text.*heapless*vec*Vec*remove*)

} > ROTEXT