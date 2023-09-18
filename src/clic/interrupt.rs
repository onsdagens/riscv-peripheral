//! Interrupt register control for a CLIC

use crate::{
    common::{Reg, RW},
    clic::InterruptNumber, //this interruptnumber should maybe be a general thing...
};
/// In a CLIC, all properties of an interrupt are controlled via a single
/// word-wide register block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct INTERRUPTS{
    ptr: *mut u32,
}

impl INTERRUPTS {
    #[inline]
    pub(crate) const unsafe fn new(address: usize) -> Self{
        Self {ptr: address as _}
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn address(self) -> usize {
        self.ptr as _
    }

    #[inline]
    pub fn is_enabled<I: InterruptNumber>(self, source: I) -> bool {
       true 
    }
}
