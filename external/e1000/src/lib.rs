#![no_std]
#![allow(unused)]

extern crate alloc;

#[macro_use]
extern crate log;

pub mod e1000;
pub mod pci;
mod utils;
mod volatile;

pub use volatile::Volatile;

pub trait Ext {}
