#![no_std]

use core::ptr;

/// A wrapper type around a volatile variable, which allows for volatile reads
/// and writes to the contained value. The stored type needs to be `Copy`, as
/// volatile reads and writes take and return copies of the value.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Volatile<T: Copy>(T);

impl<T: Copy> Volatile<T> {
    pub const fn new(value: T) -> Volatile<T> {
        Volatile(value)
    }

    pub fn read(&self) -> T {
        unsafe { ptr::read_volatile(&self.0) }
    }

    pub fn write(&mut self, value: T) {
        unsafe { ptr::write_volatile(&mut self.0, value) };
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

impl<T: Copy> Clone for Volatile<T> {
    fn clone(&self) -> Self {
        Volatile(self.read())
    }
}

pub type ReadWrite<T> = Volatile<T>;
