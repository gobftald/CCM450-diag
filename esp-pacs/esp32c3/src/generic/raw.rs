use super::{marker, FieldSpec, RegisterSpec, Unsafe, Writable};

// 2
pub struct R<REG: RegisterSpec> {
    pub(crate) bits: REG::Ux,
    pub(super) _reg: marker::PhantomData<REG>,
}

// 6
pub struct W<REG: RegisterSpec> {
    /// Writable bits
    pub(crate) bits: REG::Ux,
    pub(super) _reg: marker::PhantomData<REG>,
}

// 44
pub struct FieldWriter<'a, REG, const WI: u8, FI = u8, Safety = Unsafe>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
{
    pub(crate) w: &'a mut W<REG>,
    pub(crate) o: u8,
    _field: marker::PhantomData<(FI, Safety)>,
}

// 53
impl<'a, REG, const WI: u8, FI, Safety> FieldWriter<'a, REG, WI, FI, Safety>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
{
    #[doc = " Creates a new instance of the writer"]
    #[allow(unused)]
    #[inline(always)]
    pub(crate) fn new(w: &'a mut W<REG>, o: u8) -> Self {
        Self {
            w,
            o,
            _field: marker::PhantomData,
        }
    }
}
