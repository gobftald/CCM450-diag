/// mac intr map register
pub struct CPU_INT_ENABLE_SPEC;

impl crate::RegisterSpec for CPU_INT_ENABLE_SPEC {
    type Ux = u32;
}

/// `read()` method returns [`cpu_int_enable::R`](R) reader structure
impl crate::Readable for CPU_INT_ENABLE_SPEC {}

/// `write(|w| ..)` method takes [`cpu_int_enable::W`](W) writer structure
impl crate::Writable for CPU_INT_ENABLE_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
