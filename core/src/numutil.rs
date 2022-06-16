/// Trait for common number operations.
pub trait NumExt {
    /// Always the implementor itself.
    type Output;

    /// Get the state of the given bit. Returns 0/1.
    fn bit(self, bit: u16) -> Self::Output;
    /// Is the given bit set?
    fn is_bit(&self, bit: u16) -> bool;
    /// Set the given bit.
    fn set_bit(self, bit: u16, state: bool) -> Self::Output;
    /// Convert to u8
    fn u8(self) -> u8;
    /// Convert to u16
    fn u16(self) -> u16;
    /// Convert to u32
    fn u32(self) -> u32;
    /// Convert to usize
    fn us(self) -> usize;

    /// Shift to the left, giving 0 if it does not fit.
    fn wshl(self, by: u32) -> Self::Output;
    /// Shift to the right, giving 0 if it does not fit.
    fn wshr(self, by: u32) -> Self::Output;
}

macro_rules! num_ext_impl {
    ($ty:ident) => {
        impl NumExt for $ty {
            type Output = $ty;

            #[inline(always)]
            fn bit(self, bit: u16) -> $ty {
                ((self >> bit) & 1)
            }

            #[inline(always)]
            fn is_bit(&self, bit: u16) -> bool {
                (self & (1 << bit)) != 0
            }

            #[inline(always)]
            fn set_bit(self, bit: u16, state: bool) -> $ty {
                (self & ((1 << bit) ^ Self::MAX)) | ((state as $ty) << bit)
            }

            #[inline(always)]
            fn u8(self) -> u8 {
                self as u8
            }

            #[inline(always)]
            fn u16(self) -> u16 {
                self as u16
            }

            #[inline(always)]
            fn u32(self) -> u32 {
                self as u32
            }

            #[inline(always)]
            fn us(self) -> usize {
                self as usize
            }

            fn wshl(self, by: u32) -> $ty {
                self.checked_shl(by).unwrap_or(0)
            }

            fn wshr(self, by: u32) -> $ty {
                self.checked_shr(by).unwrap_or(0)
            }
        }
    };
}

num_ext_impl!(u8);
num_ext_impl!(u16);
num_ext_impl!(u32);
num_ext_impl!(u64);
num_ext_impl!(usize);

// Traits and functions for some more common operations used mainly on GGA.
pub fn hword(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) | lo as u16
}

pub fn word(lo: u16, hi: u16) -> u32 {
    ((hi as u32) << 16) | lo as u32
}

pub trait U16Ext {
    fn low(self) -> u8;
    fn high(self) -> u8;
    fn set_low(self, low: u8) -> u16;
    fn set_high(self, high: u8) -> u16;
    fn bits(self, start: u16, len: u16) -> u16;
    fn i10(self) -> i16;
}

impl U16Ext for u16 {
    fn low(self) -> u8 {
        self.u8()
    }

    fn high(self) -> u8 {
        (self >> 8).u8()
    }

    fn set_low(self, low: u8) -> u16 {
        (self & 0xFF00) | low.u16()
    }

    fn set_high(self, high: u8) -> u16 {
        (self & 0x00FF) | (high.u16() << 8)
    }

    fn bits(self, start: u16, len: u16) -> u16 {
        (self >> start) & ((1 << len) - 1)
    }

    fn i10(self) -> i16 {
        let mut result = self & 0x3FF;
        if (self & 0x0400) > 1 {
            result |= 0xFC00;
        }
        result as i16
    }
}

pub trait U32Ext {
    fn low(self) -> u16;
    fn high(self) -> u16;
    fn set_low(self, low: u16) -> u32;
    fn set_high(self, high: u16) -> u32;
    fn bits(self, start: u32, len: u32) -> u32;
    fn i24(self) -> i32;
}

impl U32Ext for u32 {
    fn low(self) -> u16 {
        self.u16()
    }

    fn high(self) -> u16 {
        (self >> 16).u16()
    }

    fn set_low(self, low: u16) -> u32 {
        (self & 0xFFFF0000) | low.u32()
    }

    fn set_high(self, high: u16) -> u32 {
        (self & 0x0000FFFF) | (high.u32() << 16)
    }

    fn bits(self, start: u32, len: u32) -> u32 {
        (self >> start) & ((1 << len) - 1)
    }

    fn i24(self) -> i32 {
        let mut result = self & 0xFF_FFFF;
        if (self & 0x80_0000) > 0 {
            result |= 0xFF00_0000;
        }
        result as i32
    }
}