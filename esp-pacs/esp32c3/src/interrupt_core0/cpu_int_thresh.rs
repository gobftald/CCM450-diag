/// Register `CPU_INT_THRESH` reader
pub type R = crate::R<CPU_INT_THRESH_SPEC>;

/// mac intr map register
pub struct CPU_INT_THRESH_SPEC;

impl crate::RegisterSpec for CPU_INT_THRESH_SPEC {
    type Ux = u32;
}

/// `read()` method returns [`cpu_int_thresh::R`](R) reader structure
impl crate::Readable for CPU_INT_THRESH_SPEC {}

/// `write(|w| ..)` method takes [`cpu_int_thresh::W`](W) writer structure
impl crate::Writable for CPU_INT_THRESH_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

/// `reset()` method sets CPU_INT_THRESH to value 0
impl crate::Resettable for CPU_INT_THRESH_SPEC {
    const RESET_VALUE: u32 = 0;
}
