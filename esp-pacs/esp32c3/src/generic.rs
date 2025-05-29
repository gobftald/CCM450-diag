use core::marker;

/// Raw register type (`u8`, `u16`, `u32`, ...)
// 3
pub trait RawReg:
    Copy
    + Default
    + From<bool>
    + core::ops::BitOr<Output = Self>
    + core::ops::BitAnd<Output = Self>
    + core::ops::BitOrAssign
    + core::ops::BitAndAssign
    + core::ops::Not<Output = Self>
    + core::ops::Shl<u8, Output = Self>
{
    /// Mask for bits of width `WI`
    fn mask<const WI: u8>() -> Self;
    /// Mask for bits of width 1
    fn one() -> Self;
}

// 19
macro_rules! raw_reg {
    ($ U : ty , $ size : literal , $ mask : ident) => {
        // 21
        impl RawReg for $U {
            #[inline(always)]
            fn mask<const WI: u8>() -> Self {
                $mask::<WI>()
            }
            #[inline(always)]
            fn one() -> Self {
                1
            }
        }

        // 31
        const fn $mask<const WI: u8>() -> $U {
            <$U>::MAX >> ($size - WI)
        }

        // 34
        impl FieldSpec for $U {
            type Ux = $U;
        }
    };
}

// 39
raw_reg!(u8, 8, mask_u8);
raw_reg!(u16, 16, mask_u16);
raw_reg!(u32, 32, mask_u32);
raw_reg!(u64, 64, mask_u64);

/// Raw register type
// 44
pub trait RegisterSpec {
    /// Raw register type (`u8`, `u16`, `u32`, ...)."]
    type Ux: RawReg;
}

/// Raw field type
pub trait FieldSpec: Sized {
    /// Raw field type (`u8`, `u16`, `u32`, ...).
    type Ux: Copy + /*core::fmt::Debug +*/ PartialEq + From<Self>;
}

/// Trait implemented by readable registers to enable the `read` method.
///
/// Registers marked with `Writable` can be also be `modify`'ed.
// 58
pub trait Readable: RegisterSpec {}

/// Trait implemented by writeable registers.
///
/// This enables the  `write`, `write_with_zero` and `reset` methods.
///
/// Registers marked with `Readable` can be also be `modify`'ed.
// 64
pub trait Writable: RegisterSpec {
    // Is it safe to write any bits to register
    type Safety;
    /// Specifies the register bits that are not changed if you pass `1` and are changed if you pass `0`
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux;
    // Specifies the register bits that are not changed if you pass `0` and are changed if you pass `1`
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux;
}

/// Reset value of the register.
///
/// This value is the initial value for the `write` method. It can also be directly written to the
/// register by using the `reset` method.
// 76
pub trait Resettable: RegisterSpec {
    /// Reset value of the register.
    const RESET_VALUE: Self::Ux;
    /// Reset value of the register.
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        Self::RESET_VALUE
    }
}

// 86
pub mod raw;

/// Register reader.
///
/// Result of the `read` methods of registers. Also used as a closure argument in the `modify`
/// method.
// 91
pub type R<REG> = raw::R<REG>;

// 92
impl<REG: RegisterSpec> R<REG> {
    /// Reads raw bits from register.
    #[inline(always)]
    pub const fn bits(&self) -> REG::Ux {
        self.bits
    }
}

/// Register writer.
///
/// Used as an argument to the closures in the `write` and `modify` methods of the register.
// 113
pub type W<REG> = raw::W<REG>;

// 114
impl<REG: Writable> W<REG> {
    /// Writes raw bits to the register.
    ///
    /// # Safety
    ///
    /// Passing incorrect value can cause undefined behaviour. See reference manual
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: REG::Ux) -> &mut Self {
        self.bits = bits;
        self
    }
}

/// Field reader.
///
/// Result of the `read` methods of fields.
pub type FieldReader<FI = u8> = raw::FieldReader<FI>;

/// Bit-wise field reader
// 142
pub type BitReader<FI = bool> = raw::BitReader<FI>;

// 143
impl<FI: FieldSpec> FieldReader<FI> {
    /// Reads raw bits from field.
    #[inline(always)]
    pub const fn bits(&self) -> FI::Ux {
        self.bits
    }
}

// 174
impl<FI> BitReader<FI> {
    /// Value of the field as raw bits.
    #[inline(always)]
    // 177
    pub const fn bit(&self) -> bool {
        self.bits
    }

    /// Returns `true` if the bit is set (1).
    #[inline(always)]
    // 187
    pub const fn bit_is_set(&self) -> bool {
        self.bit()
    }
}

/// Marker for register/field writers which can take any value of specified width
// 197
pub struct Safe;

/// You should check that value is allowed to pass to register/field writer marked with this
// 199
pub struct Unsafe;

/// Write field Proxy
// 207
pub type FieldWriter<'a, REG, const WI: u8, FI = u8, Safety = Unsafe> =
    raw::FieldWriter<'a, REG, WI, FI, Safety>;

// 227
impl<'a, REG, const WI: u8, FI, Safety> FieldWriter<'a, REG, WI, FI, Safety>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
    REG::Ux: From<FI::Ux>,
{
    /// Writes raw bits to the field
    ///
    /// # Safety
    ///
    /// Passing incorrect value can cause undefined behaviour. See reference manual
    #[inline(always)]
    pub unsafe fn bits(self, value: FI::Ux) -> &'a mut W<REG> {
        self.w.bits &= !(REG::Ux::mask::<WI>() << self.o);
        self.w.bits |= (REG::Ux::from(value) & REG::Ux::mask::<WI>()) << self.o;
        self.w
    }
}

// 245
impl<'a, REG, const WI: u8, FI> FieldWriter<'a, REG, WI, FI, Safe>
where
    REG: Writable + RegisterSpec,
    FI: FieldSpec,
    REG::Ux: From<FI::Ux>,
{
    /// Writes raw bits to the field
    #[inline(always)]
    pub fn set(self, value: FI::Ux) -> &'a mut W<REG> {
        unsafe { self.bits(value) }
    }
}

// 321
macro_rules! bit_proxy {
    ($ writer : ident , $ mwv : ident) => {
        pub struct $mwv;
        /// Bit-wise write field proxy
        pub type $writer<'a, REG, FI = bool> = raw::BitWriter<'a, REG, FI, $mwv>;
        impl<'a, REG, FI> $writer<'a, REG, FI>
        where
            REG: Writable + RegisterSpec,
            bool: From<FI>,
        {
            #[doc = " Writes bit to the field"]
            #[inline(always)]
            pub fn bit(self, value: bool) -> &'a mut W<REG> {
                self.w.bits &= !(REG::Ux::one() << self.o);
                self.w.bits |= (REG::Ux::from(value) & REG::Ux::one()) << self.o;
                self.w
            }
        }
    };
}

// 359
bit_proxy!(BitWriter, BitM);
// 362
bit_proxy!(BitWriter1C, Bit1C);

// 366
impl<'a, REG, FI> BitWriter<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    /// Sets the field bit
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W<REG> {
        self.w.bits |= REG::Ux::one() << self.o;
        self.w
    }

    /// Clears the field bit
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W<REG> {
        self.w.bits &= !(REG::Ux::one() << self.o);
        self.w
    }
}

// 408
impl<'a, REG, FI> BitWriter1C<'a, REG, FI>
where
    REG: Writable + RegisterSpec,
    bool: From<FI>,
{
    /// Clears the field bit by passing one
    #[inline(always)]
    pub fn clear_bit_by_one(self) -> &'a mut W<REG> {
        self.w.bits |= REG::Ux::one() << self.o;
        self.w
    }
}

/// This structure provides volatile access to registers.
#[repr(transparent)]
// 458
pub struct Reg<REG: RegisterSpec> {
    register: vcell::VolatileCell<REG::Ux>,
    _marker: marker::PhantomData<REG>,
}

// 474
impl<REG: Readable> Reg<REG> {
    /// Reads the contents of a `Readable` register.
    ///
    /// You can read the raw contents of a register by using `bits`:
    /// ```ignore
    /// let bits = periph.reg.read().bits();
    /// ```
    /// or get the content of a particular field of a register:
    /// ```ignore
    /// let reader = periph.reg.read();
    /// let bits = reader.field1().bits();
    /// let flag = reader.field2().bit_is_set();
    /// ```
    #[inline(always)]
    pub fn read(&self) -> R<REG> {
        R {
            bits: self.register.get(),
            _reg: marker::PhantomData,
        }
    }
}

// 495
impl<REG: Resettable + Writable> Reg<REG> {
    /// Writes bits to a `Writable` register.
    ///
    /// You can write raw bits into a register:
    /// ```ignore
    /// periph.reg.write(|w| unsafe { w.bits(rawbits) });
    /// ```
    /// or write only the fields you need:
    /// ```ignore
    /// periph.reg.write(|w| w
    ///     .field1().bits(newfield1bits)
    ///     .field2().set_bit()
    ///     .field3().variant(VARIANT)
    /// );
    /// ```
    /// or an alternative way of saying the same:
    /// ```ignore
    /// periph.reg.write(|w| {
    ///     w.field1().bits(newfield1bits);
    ///     w.field2().set_bit();
    ///     w.field3().variant(VARIANT)
    /// });
    /// ```
    /// In the latter case, other fields will be set to their reset value.
    #[inline(always)]
    // 527
    pub fn write<F>(&self, f: F) -> REG::Ux
    where
        F: FnOnce(&mut W<REG>) -> &mut W<REG>,
    {
        let value = f(&mut W {
            bits: REG::RESET_VALUE & !REG::ONE_TO_MODIFY_FIELDS_BITMAP
                | REG::ZERO_TO_MODIFY_FIELDS_BITMAP,
            _reg: marker::PhantomData,
        })
        .bits;
        self.register.set(value);
        value
    }
}

// 625
impl<REG: Readable + Writable> Reg<REG> {
    /// Modifies the contents of the register by reading and then writing it.
    ///
    /// E.g. to do a read-modify-write sequence to change parts of a register:
    /// ```ignore"]
    /// periph.reg.modify(|r, w| unsafe { w.bits(
    ///    r.bits() | 3
    /// ) });
    /// ```
    /// or
    /// ```ignore
    /// periph.reg.modify(|_, w| w
    ///     .field1().bits(newfield1bits)
    ///     .field2().set_bit()
    ///     .field3().variant(VARIANT)
    /// );
    /// ```
    /// or an alternative way of saying the same:
    /// ```ignore
    /// periph.reg.modify(|_, w| {
    ///     w.field1().bits(newfield1bits);
    ///     w.field2().set_bit();
    ///     w.field3().variant(VARIANT)
    /// });
    /// ```
    /// Other fields will have the value they had before the call to `modify`.
    #[inline(always)]
    pub fn modify<F>(&self, f: F) -> REG::Ux
    where
        for<'w> F: FnOnce(&R<REG>, &'w mut W<REG>) -> &'w mut W<REG>,
    {
        let bits = self.register.get();
        let value = f(
            &R {
                bits,
                _reg: marker::PhantomData,
            },
            &mut W {
                bits: bits & !REG::ONE_TO_MODIFY_FIELDS_BITMAP | REG::ZERO_TO_MODIFY_FIELDS_BITMAP,
                _reg: marker::PhantomData,
            },
        )
        .bits;
        self.register.set(value);
        value
    }
}
