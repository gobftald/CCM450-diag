use super::{marker, BitM, FieldSpec, RegisterSpec, Unsafe, Writable};

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

// 11
pub struct FieldReader<FI = u8>
where
    FI: FieldSpec,
{
    pub(crate) bits: FI::Ux,
    _reg: marker::PhantomData<FI>,
}

// 18
impl<FI: FieldSpec> FieldReader<FI> {
    // Creates a new instance of the reader.
    #[allow(unused)]
    #[inline(always)]
    pub(crate) const fn new(bits: FI::Ux) -> Self {
        Self {
            bits,
            _reg: marker::PhantomData,
        }
    }
}

// 29
pub struct BitReader<FI = bool> {
    pub(crate) bits: bool,
    _reg: marker::PhantomData<FI>,
}

// 33
impl<FI> BitReader<FI> {
    ///Creates a new instance of the reader.
    #[allow(unused)]
    #[inline(always)]
    pub(crate) const fn new(bits: bool) -> Self {
        Self {
            bits,
            _reg: marker::PhantomData,
        }
    }
}

// 45
pub struct FieldWriter<'a, REG, const WI: u8, FI = u8, Safety = Unsafe>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
{
    pub(crate) w: &'a mut W<REG>,
    pub(crate) o: u8,
    _field: marker::PhantomData<(FI, Safety)>,
}

// 54
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

#[must_use = "after creating `BitWriter` you need to call bit setting method"]
// 71
pub struct BitWriter<'a, REG, FI = bool, M = BitM>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    pub(crate) w: &'a mut W<REG>,
    pub(crate) o: u8,
    _field: marker::PhantomData<(FI, M)>,
}

// 80
impl<'a, REG, FI, M> BitWriter<'a, REG, FI, M>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    /// Creates a new instance of the writer
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
