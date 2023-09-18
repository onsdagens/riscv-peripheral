//! Interrupt register control for a CLIC

use crate::{
    common::{Reg, RW},
    clic::{InterruptNumber, PriorityNumber}, //this interruptnumber should maybe be a general thing...
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
    
    /// Checks if an interrupt source is enabled.
    #[inline]
    pub fn is_enabled<I: InterruptNumber>(self, source: I) -> bool {
        let source = source.number() as usize;
        let offset = (source * 4 + 1) as _;

        let reg: Reg<u8, RW> = unsafe { Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.read() == 1
    }

    /// Enables an interrupt source.
    ///
    /// # Safety
    ///
    /// * Enabling an interrupt source can break mask-based critical sections.
    #[inline]
    pub unsafe fn enable<I: InterruptNumber>(self, source: I) {
        let source = source.number() as usize;
        let offset = (source * 4 + 1) as _;
        // SAFETY: valid interrupt number
        let reg: Reg<u8, RW> = unsafe { Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.write(1);
    }

    /// Disables an interrupts source.
    pub fn disable<I: InterruptNumber>(self, source: I) {
        let source = source.number() as usize;
        let offset = (source * 4 + 1) as _;
        // SAFETY: valid interrupt number
        let reg: Reg<u8, RW> = unsafe { Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.write(0);
    }

    /// Returns the configured priority of an interrupt source
    #[inline]
    pub fn get_priority<I: InterruptNumber>(self, source: I) -> u8 {
        let source = source.number() as usize;
        let offset = (source * 4 + 3) as _;
        // SAFETY: valid interrupt number
        let reg: Reg<u8, RW> = unsafe { Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.read()
    }
    /// Sets the priority of an interrupt source
    /// 
    /// # Safety
    ///
    /// * Changing/setting the priority of an interrupt may break mask-based critical sections.
    #[inline]
    pub unsafe fn set_priority<I: InterruptNumber, P: PriorityNumber>(self, source: I, prio: P) {
        let source = source.number() as usize;
        let offset = (source * 4 + 3) as _;
        let prio = prio.number();
        // SAFETY: valid interrupt number
        let reg: Reg<u8, RW> = unsafe {Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.write(prio);
    }

    /// Retuns the pending status of an interrupt
    #[inline]
    pub fn is_pending<I: InterruptNumber>(self, source: I) -> bool {
        let source = source.number() as usize;
        let offset= (source*4) as _;
        // SAFETY: valid interrupt number
        let reg: Reg<u8, RW> = unsafe{ Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.read() == 1
    }

    /// Sets an interrupt as pending
    ///
    /// # Safety
    ///
    /// * Pending interrupts may break mask-based critical sections.
    #[inline]
    pub unsafe fn pend<I: InterruptNumber>(self, source: I) {
        let source = source.number() as usize;
        let offset = (source*4) as _;
        // SAFETY: valid interrupt number
        let reg: Reg<u8, RW> = unsafe{ Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.write(1);
    }

    /// Sets an interrupt as not pending
    #[inline]
    pub fn unpend<I: InterruptNumber>(self, source: I) {
        let source = source.number() as usize;
        let offset = (source*4) as _;
        // SAFETY: valid interrupt number
        let reg: Reg<u8, RW> = unsafe{ Reg::new(self.ptr.offset(offset) as *mut u8)};
        reg.write(0);
    }
}

#[cfg(test)]
mod test{
    use super::super::test::Interrupt;
    use super::*;

    #[test]
    fn test_enables() {
        let mut raw_reg = [0u32; 32];

        let interrupts = unsafe { INTERRUPTS::new(raw_reg.as_mut_ptr() as _)};

        unsafe{interrupts.enable(Interrupt::I1)};
        unsafe{interrupts.enable(Interrupt::I2)};
        unsafe{interrupts.enable(Interrupt::I3)};
        unsafe{interrupts.enable(Interrupt::I4)};
        interrupts.disable(Interrupt::I2);
        interrupts.disable(Interrupt::I4);
        assert!(interrupts.is_enabled(Interrupt::I1));
        assert!(!interrupts.is_enabled(Interrupt::I2));
        assert!(interrupts.is_enabled(Interrupt::I3));
        assert!(!interrupts.is_enabled(Interrupt::I4));
    }
}
