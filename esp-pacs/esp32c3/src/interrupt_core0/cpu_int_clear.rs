/// mac intr map register
pub struct CPU_INT_CLEAR_SPEC;

impl crate::RegisterSpec for CPU_INT_CLEAR_SPEC {
    type Ux = u32;
}

/// `write(|w| ..)` method takes [`cpu_int_clear::W`](W) writer structure
impl crate::Writable for CPU_INT_CLEAR_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

/// `reset()` method sets CPU_INT_CLEAR to value 0
impl crate::Resettable for CPU_INT_CLEAR_SPEC {
    const RESET_VALUE: u32 = 0;
}
