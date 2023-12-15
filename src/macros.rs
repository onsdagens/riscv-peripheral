//! Utility macros for generating standard peripherals-related code in RISC-V PACs.

/// Macro to create interfaces to CLINT peripherals in PACs.
/// The resulting struct will be named `CLINT`, and will provide safe access to the CLINT registers.
///
/// This macro expects 4 different argument types:
///
/// - Base address (**MANDATORY**): base address of the CLINT peripheral of the target.
/// - Frequency (**OPTIONAL**): clock frequency (in Hz) of the `MTIME` register. It enables the `delay` method of the `CLINT` struct.
/// - Per-HART mtimecmp registers (**OPTIONAL**): a list of `mtimecmp` registers for easing access to per-HART mtimecmp regs.
/// - Per-HART msip registers (**OPTIONAL**): a list of `msip` registers for easing access to per-HART msip regs.
///
/// Check the examples below for more details about the usage and syntax of this macro.
///
/// # Example
///
/// ## Base address only
///
/// ```
/// use riscv_peripheral::clint_codegen;
///
/// clint_codegen!(base 0x0200_0000, freq 32_768,); // do not forget the ending comma!
///
/// let mswi = CLINT::mswi(); // MSWI peripheral
/// let mtimer = CLINT::mtimer(); // MTIMER peripheral
/// let delay = CLINT::delay(); // For the `embedded_hal::delay::DelayUs` and `embedded_hal_async::delay::DelayUs` traits
/// ```
///
/// ## Base address and per-HART mtimecmp registers
///
/// ```
/// use riscv_peripheral::clint_codegen;
///
/// /// HART IDs for the target CLINT peripheral
/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// #[repr(u16)]
/// pub enum HartId { H0 = 0, H1 = 1, H2 = 2 }
///
/// // Implement `HartIdNumber` for `HartId`
/// unsafe impl riscv_peripheral::aclint::HartIdNumber for HartId {
///   const MAX_HART_ID_NUMBER: u16 = 2;
///   fn number(self) -> u16 { self as _ }
///   fn from_number(number: u16) -> Result<Self, u16> {
///     if number > Self::MAX_HART_ID_NUMBER {
///        Err(number)
///     } else {
///        // SAFETY: valid context number
///        Ok(unsafe { core::mem::transmute(number) })
///     }
///   }
/// }
///
/// clint_codegen!(
///     base 0x0200_0000,
///     mtimecmps [mtimecmp0 = (HartId::H0, "`H0`"), mtimecmp1 = (HartId::H1, "`H1`"), mtimecmp2 = (HartId::H2, "`H2`")],
///     msips [msip0=(HartId::H0,"`H0`"), msip1=(HartId::H1,"`H1`"), msip2=(HartId::H2,"`H2`")], // do not forget the ending comma!
/// );
///
/// let mswi = CLINT::mswi(); // MSWI peripheral
/// let mtimer = CLINT::mtimer(); // MTIMER peripheral
///
/// let mtimecmp0 = CLINT::mtimecmp0(); // mtimecmp register for HART 0
/// let mtimecmp1 = CLINT::mtimecmp1(); // mtimecmp register for HART 1
/// let mtimecmp2 = CLINT::mtimecmp2(); // mtimecmp register for HART 2
///
/// let msip0 = CLINT::msip0(); // msip register for HART 0
/// let msip1 = CLINT::msip1(); // msip register for HART 1
/// let msip2 = CLINT::msip2(); // msip register for HART 2
/// ```
#[macro_export]
macro_rules! clint_codegen {
    () => {
        #[allow(unused_imports)]
        use CLINT as _; // assert that the CLINT struct is defined
    };
    (base $addr:literal, $($tail:tt)*) => {
        /// CLINT peripheral
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct CLINT;

        unsafe impl $crate::aclint::Clint for CLINT {
            const BASE: usize = $addr;
        }

        impl CLINT {
            /// Returns `true` if a machine timer **OR** software interrupt is pending.
            #[inline]
            pub fn is_interrupting() -> bool {
                Self::mswi_is_interrupting() || Self::mtimer_is_interrupting()
            }

            /// Returns `true` if machine timer **OR** software interrupts are enabled.
            pub fn is_enabled() -> bool {
                Self::mswi_is_enabled() || Self::mtimer_is_enabled()
            }

            /// Enables machine timer **AND** software interrupts to allow the CLINT to trigger interrupts.
            ///
            /// # Safety
            ///
            /// Enabling the `CLINT` may break mask-based critical sections.
            #[inline]
            pub unsafe fn enable() {
                Self::mswi_enable();
                Self::mtimer_enable();
            }

            /// Disables machine timer **AND** software interrupts to prevent the CLINT from triggering interrupts.
            #[inline]
            pub fn disable() {
                Self::mswi_disable();
                Self::mtimer_disable();
            }

            /// Returns `true` if a machine software interrupt is pending.
            #[inline]
            pub fn mswi_is_interrupting() -> bool {
                $crate::riscv::register::mip::read().msoft()
            }

            /// Returns `true` if Machine Software Interrupts are enabled.
            #[inline]
            pub fn mswi_is_enabled() -> bool {
                $crate::riscv::register::mie::read().msoft()
            }

            /// Enables the `MSWI` peripheral.
            ///
            /// # Safety
            ///
            /// Enabling the `MSWI` may break mask-based critical sections.
            #[inline]
            pub unsafe fn mswi_enable() {
                $crate::riscv::register::mie::set_msoft();
            }

            /// Disables the `MSWI` peripheral.
            #[inline]
            pub fn mswi_disable() {
                // SAFETY: it is safe to disable interrupts
                unsafe { $crate::riscv::register::mie::clear_msoft() };
            }

            /// Returns the `MSWI` peripheral.
            #[inline]
            pub const fn mswi() -> $crate::aclint::mswi::MSWI {
                $crate::aclint::CLINT::<CLINT>::mswi()
            }

            /// Returns `true` if a machine timer interrupt is pending.
            #[inline]
            pub fn mtimer_is_interrupting() -> bool {
                $crate::riscv::register::mip::read().mtimer()
            }

            /// Returns `true` if Machine Timer Interrupts are enabled.
            #[inline]
            pub fn mtimer_is_enabled() -> bool {
                $crate::riscv::register::mie::read().mtimer()
            }

            /// Sets the Machine Timer Interrupt bit of the `mie` CSR.
            /// This bit must be set for the `MTIMER` to trigger machine timer interrupts.
            ///
            /// # Safety
            ///
            /// Enabling the `MTIMER` may break mask-based critical sections.
            #[inline]
            pub unsafe fn mtimer_enable() {
                $crate::riscv::register::mie::set_mtimer();
            }

            /// Clears the Machine Timer Interrupt bit of the `mie` CSR.
            /// When cleared, the `MTIMER` cannot trigger machine timer interrupts.
            #[inline]
            pub fn mtimer_disable() {
                // SAFETY: it is safe to disable interrupts
                unsafe { $crate::riscv::register::mie::clear_mtimer() };
            }

            /// Returns the `MTIMER` peripheral.
            #[inline]
            pub const fn mtimer() -> $crate::aclint::mtimer::MTIMER {
                $crate::aclint::CLINT::<CLINT>::mtimer()
            }

            /// Returns the `MTIME` register of the `MTIMER` peripheral.
            #[inline]
            pub const fn mtime() -> $crate::aclint::mtimer::MTIME {
                Self::mtimer().mtime
            }
        }
        $crate::clint_codegen!($($tail)*);
    };
    (freq $freq:literal, $($tail:tt)*) => {
        impl CLINT {
            /// Returns the frequency of the `MTIME` register.
            #[inline]
            pub const fn freq() -> usize {
                $freq
            }

            /// Delay implementation for CLINT peripherals.
            ///
            /// # Note
            ///
            /// You must export the `riscv_peripheral::hal::delay::DelayUs` trait in order to use delay methods.
            /// You must export the `riscv_peripheral::hal_async::delay::DelayUs` trait in order to use async delay methods.
            #[inline]
            pub const fn delay() -> $crate::hal::aclint::Delay {
                $crate::hal::aclint::Delay::new(Self::mtime(), Self::freq())
            }
        }
        $crate::clint_codegen!($($tail)*);
    };
    (msips [$($fn:ident = ($hart:expr , $shart:expr)),+], $($tail:tt)*) => {
        impl CLINT {
            $(
                #[doc = "Returns the `msip` register for HART "]
                #[doc = $shart]
                #[doc = "."]
                #[inline]
                pub fn $fn() -> $crate::aclint::mswi::MSIP {
                    Self::mswi().msip($hart)
                }
            )*
        }
        $crate::clint_codegen!($($tail)*);
    };
    (mtimecmps [$($fn:ident = ($hart:expr , $shart:expr)),+], $($tail:tt)*) => {
        impl CLINT {
            $(
                #[doc = "Returns the `mtimecmp` register for HART "]
                #[doc = $shart]
                #[doc = "."]
                #[inline]
                pub fn $fn() -> $crate::aclint::mtimer::MTIMECMP {
                    Self::mtimer().mtimecmp($hart)
                }
            )*
        }
        $crate::clint_codegen!($($tail)*);
    };
}

/// Macro to create interfaces to PLIC peripherals in PACs.
#[macro_export]
macro_rules! plic_codegen {
    () => {
        #[allow(unused_imports)]
        use PLIC as _; // assert that the PLIC struct is defined
    };
    (base $addr:literal, $($tail:tt)*) => {
        /// PLIC peripheral
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct PLIC;

        unsafe impl $crate::plic::Plic for PLIC {
            const BASE: usize = $addr;
        }

        impl PLIC {
            /// Returns `true` if a machine external interrupt is pending.
            #[inline]
            pub fn is_interrupting() -> bool {
                $crate::riscv::register::mip::read().mext()
            }

            /// Returns true if Machine External Interrupts are enabled.
            #[inline]
            pub fn is_enabled() -> bool {
                $crate::riscv::register::mie::read().mext()
            }

            /// Enables machine external interrupts to allow the PLIC to trigger interrupts.
            ///
            /// # Safety
            ///
            /// Enabling the `PLIC` may break mask-based critical sections.
            #[inline]
            pub unsafe fn enable() {
                $crate::riscv::register::mie::set_mext();
            }

            /// Disables machine external interrupts to prevent the PLIC from triggering interrupts.
            #[inline]
            pub fn disable() {
                // SAFETY: it is safe to disable interrupts
                unsafe { $crate::riscv::register::mie::clear_mext() };
            }

            /// Returns the priorities register of the PLIC.
            #[inline]
            pub fn priorities() -> $crate::plic::priorities::PRIORITIES {
                $crate::plic::PLIC::<PLIC>::priorities()
            }

            /// Returns the pendings register of the PLIC.
            #[inline]
            pub fn pendings() -> $crate::plic::pendings::PENDINGS {
                $crate::plic::PLIC::<PLIC>::pendings()
            }

            /// Returns the context proxy of a given PLIC context.
            #[inline]
            pub fn ctx<C: $crate::plic::ContextNumber>(context: C) -> $crate::plic::CTX<Self> {
                $crate::plic::PLIC::<PLIC>::ctx(context)
            }
        }
        $crate::plic_codegen!($($tail)*);
    };
    (ctxs [$($fn:ident = ($ctx:expr , $sctx:expr)),+], $($tail:tt)*) => {
        impl PLIC {
            $(
                #[doc = "Returns a PLIC context proxy for context "]
                #[doc = $sctx]
                #[doc = "."]
                #[inline]
                pub fn $fn() -> $crate::plic::CTX<Self> {
                    Self::ctx($ctx)
                }
            )*
        }
        $crate::plic_codegen!($($tail)*);
    };
}

#[macro_export]
macro_rules! clic_codegen {
    () => {
        #[allow(unused_imports)]
        use CLIC as _; // assert that the PLIC struct is defined
    };
    (base $addr:literal) => {
        /// PLIC peripheral
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct CLIC;

        unsafe impl $crate::clic::Clic for CLIC {
            const BASE: usize = $addr;
        }

        impl CLIC {
            /// Sets the Machine Mode Interrupt Enable bit of the `mstatus` CSR.
            /// When set, CLIC interrupts are effectively enabled.
            ///
            /// # Safety
            ///
            /// Enabling interrupts may break critical sections.
            #[inline]
            pub unsafe fn enable() {
                $crate::clic::CLIC::<CLIC>::enable();
            }

            /// Clears the Machine Mode Interrupt Enable bit of the `mstatus` CSR.
            /// When cleared, CLIC interrupts are effectively disabled.
            #[inline]
            pub fn disable() {
                $crate::clic::CLIC::<CLIC>::disable();
            }
            /// Sets the current global interrupt threshold via the mintthresh register.
            /// When set, any pending interrupt will be filtered against the threshold
            /// 
            /// # Safety
            /// Changing the threshold is side-effectful and may cause an interrupt to be
            /// inadvertently taken
            #[inline]
            pub unsafe fn set_threshold(thresh: usize) {
                $crate::clic::CLIC::<CLIC>::set_threshold(thresh);
            }
            /// Gets the current global interrupt threshold. 
            #[inline]
            pub fn get_threshold() -> usize {
                $crate::clic::CLIC::<CLIC>::get_threshold()
            }
            /// Returns the interrupt control register block of the CLIC
            #[inline]
            pub fn interrupts() -> $crate::clic::interrupt::INTERRUPTS {
                $crate::clic::CLIC::<CLIC>::interrupts()
            }
        }
    };
}
