/// mac intr map register
pub struct CPU_INT_TYPE_SPEC;

impl crate::RegisterSpec for CPU_INT_TYPE_SPEC {
    type Ux = u32;
}

/// `read()` method returns [`cpu_int_type::R`](R) reader structure
impl crate::Readable for CPU_INT_TYPE_SPEC {}

/// `write(|w| ..)` method takes [`cpu_int_type::W`](W) writer structure
impl crate::Writable for CPU_INT_TYPE_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
