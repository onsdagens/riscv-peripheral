//! Core-Local Interrupt Controller (CLIC) peripheral.
//!
//! Specification: <https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc>

pub mod interrupt;
/// Trait for enums of interrupt numbers.
///
/// This trait should be implemented by a peripheral access crate (PAC)
/// on its enum of available external interrupts for a specific device.
/// Each variant must convert to a `u16` of its interrupt number.
///
/// # Note
///
/// Recall that the interrupt number `0` is reserved as "no interrupt".
///
/// # Safety
///
/// * This trait must only be implemented on a PAC of a target with a PLIC peripheral.
/// * This trait must only be implemented on enums of external interrupts.
/// * Each enum variant must represent a distinct value (no duplicates are permitted),
/// * Each enum variant must always return the same value (do not change at runtime).
/// * All the interrupt numbers must be less than or equal to `MAX_INTERRUPT_NUMBER`.
/// * `MAX_INTERRUPT_NUMBER` must coincide with the highest allowed interrupt number.
pub unsafe trait InterruptNumber: Copy {
    /// Highest number assigned to an interrupt source.
    const MAX_INTERRUPT_NUMBER: u16;

    /// Converts an interrupt source to its corresponding number.
    fn number(self) -> u16;

    /// Tries to convert a number to a valid interrupt source.
    /// If the conversion fails, it returns an error with the number back.
    fn from_number(value: u16) -> Result<Self, u16>;
}
/// Trait for enums of priority levels.
///
/// This trait should be implemented by a peripheral access crate (PAC)
/// on its enum of available priority numbers for a specific device.
/// Each variant must convert to a `u8` of its priority level.
///
/// # Note
///
/// Recall that the priority number `0` is reserved as "never interrupt".
///
/// # Safety
///
/// * This trait must only be implemented on a PAC of a target with a PLIC peripheral.
/// * This trait must only be implemented on enums of priority levels.
/// * Each enum variant must represent a distinct value (no duplicates are permitted).
/// * Each enum variant must always return the same value (do not change at runtime).
/// * There must be a valid priority number set to 0 (i.e., never interrupt).
/// * All the priority level numbers must be less than or equal to `MAX_PRIORITY_NUMBER`.
/// * `MAX_PRIORITY_NUMBER` must coincide with the highest allowed priority number.
pub unsafe trait PriorityNumber: Copy {
    /// Platform wide highest supported priority level
    const MAX_PRIORITY_NUMBER: u8;

    /// Converts a priority level to its corresponding number.
    fn number(self) -> u8;

    /// Tries to convert a number to a valid priority level.
    /// If the conversion fails, it returns an error with the number back.
    fn from_number(value: u8) -> Result<Self, u8>;
}
/// Trait for a CLIC peripheral.
///
/// # Safety
///
/// * This trait must only be implemented on a PAC of a target with a CLIC peripheral.
/// * The CLIC peripheral base address `BASE` must be valid for the target device.
pub unsafe trait Clic: Copy {
    /// Base address of the CLIC peripheral.
    const BASE: usize;
}
/// Core-Local Interrupt Controler (CLIC) peripheral.
///
/// The RISC-V standard does not specify a fixed location for the CLIC.
/// Thus, each platform must specify the base address of the CLIC on the platform.
/// The base address, as well as all the associated types, are defined in the [`Clic`] trait.
///
/// The CLIC standard allows for up to 4096 interrupts. For each of the available interrupts, the
/// CLIC contains a register used to configure the interrupt in question. For more information,
/// refer to the (CLIC specification)[https://github.com/riscv/riscv-fast-interrupt/blob/master/clic.adoc#clic-memory-mapped-registers]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CLIC<C: Clic> {
    _marker: core::marker::PhantomData<C>,
}

impl<C: Clic> CLIC<C> {
    /// Offset to the interrupt control register block
    const INTERRUPTS_OFFSET: usize = 0x1000;

    /// Clears the Machine Mode Interrupt Enable bit of the `mstatus` CSR.
    /// When cleared, CLIC interrupts are effectively disabled.
    #[inline]
    pub fn disable() {
        // SAFETY: it is safe to disable interrupts
        unsafe { riscv::register::mstatus::clear_mie() };
    }

    /// Sets the Machine Mode Interrupt Enable bit of the `mstatus` CSR.
    /// When set, CLIC interrupts are effectively enabled.
    ///
    /// # Safety
    ///
    /// Enabling interrupts may break critical sections.
    #[inline]
    pub fn enable() {
        unsafe { riscv::register::mstatus::set_mie() };
    }
    /// Sets the global priority threshold against which all pending interrupts are filtered.
    #[inline]
    pub fn set_threshold(thresh: usize) {
        unsafe { core::arch::asm!("csrrw x0, 0x347 , {0}", in(reg) thresh as isize) };
    }
    /// Returns the current global priority threshold against which all pending interrupts are
    /// filtered.
    #[inline]
    pub fn get_threshold() -> usize {
        let r: usize;
        unsafe { core::arch::asm!(concat!("csrrs {0}, 0x347 , x0"), out(reg) r) };
        r
    }
    /// Returns the interrupt configuration registers of the CLIC.
    #[inline]
    pub fn interrupts() -> interrupt::INTERRUPTS {
        // SAFETY: Valid address
        unsafe { interrupt::INTERRUPTS::new(C::BASE + Self::INTERRUPTS_OFFSET) }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::{InterruptNumber, PriorityNumber};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u16)]
    pub(crate) enum Interrupt {
        I1 = 1,
        I2 = 2,
        I3 = 3,
        I4 = 4,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u8)]
    pub(crate) enum Priority {
        P0 = 0,
        P1 = 1,
        P2 = 2,
        P3 = 3,
    }

    unsafe impl InterruptNumber for Interrupt {
        const MAX_INTERRUPT_NUMBER: u16 = 4;

        #[inline]
        fn number(self) -> u16 {
            self as _
        }

        #[inline]
        fn from_number(number: u16) -> Result<Self, u16> {
            if number > Self::MAX_INTERRUPT_NUMBER || number == 0 {
                Err(number)
            } else {
                // SAFETY: valid priority number
                Ok(unsafe { core::mem::transmute(number) })
            }
        }
    }

    unsafe impl PriorityNumber for Priority {
        const MAX_PRIORITY_NUMBER: u8 = 3;

        #[inline]
        fn number(self) -> u8 {
            self as _
        }

        #[inline]
        fn from_number(number: u8) -> Result<Self, u8> {
            if number > Self::MAX_PRIORITY_NUMBER {
                Err(number)
            } else {
                // SAFETY: valid priority number
                Ok(unsafe { core::mem::transmute(number) })
            }
        }
    }

    #[test]
    fn check_interrupt_enum() {
        assert_eq!(Interrupt::I1.number(), 1);
        assert_eq!(Interrupt::I2.number(), 2);
        assert_eq!(Interrupt::I3.number(), 3);
        assert_eq!(Interrupt::I4.number(), 4);

        assert_eq!(Interrupt::from_number(1), Ok(Interrupt::I1));
        assert_eq!(Interrupt::from_number(2), Ok(Interrupt::I2));
        assert_eq!(Interrupt::from_number(3), Ok(Interrupt::I3));
        assert_eq!(Interrupt::from_number(4), Ok(Interrupt::I4));

        assert_eq!(Interrupt::from_number(0), Err(0));
        assert_eq!(Interrupt::from_number(5), Err(5));
    }

    #[test]
    fn check_priority_enum() {
        assert_eq!(Priority::P0.number(), 0);
        assert_eq!(Priority::P1.number(), 1);
        assert_eq!(Priority::P2.number(), 2);
        assert_eq!(Priority::P3.number(), 3);

        assert_eq!(Priority::from_number(0), Ok(Priority::P0));
        assert_eq!(Priority::from_number(1), Ok(Priority::P1));
        assert_eq!(Priority::from_number(2), Ok(Priority::P2));
        assert_eq!(Priority::from_number(3), Ok(Priority::P3));

        assert_eq!(Priority::from_number(4), Err(4));
    }

    #[allow(dead_code)]
    #[test]
    fn check_clic() {
        crate::clic_codegen!(base 0x1000);

        let interrupts = CLIC::interrupts();

        assert_eq!(interrupts.address(), 0x0000_2000);
    }
}
