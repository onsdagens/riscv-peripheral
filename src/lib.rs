//! Standard RISC-V peripherals for embedded systems written in Rust

#![deny(missing_docs)]
#![no_std]

pub use riscv; // re-export riscv crate to allow users to use it without importing it

pub mod common;
pub mod macros; // macros for easing the definition of peripherals in PACs

pub mod aclint;
pub mod plic;

pub mod clic;
