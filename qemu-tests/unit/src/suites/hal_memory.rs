extern crate alloc;

/// Heap allocator tests. The global allocator is set up by HAL init,
/// so no additional context is needed.
#[embclox_test_macros::test_suite(name = "hal_memory")]
mod tests {
    use super::*;

    /// Allocate 256 bytes with alloc_zeroed and verify all bytes are zero.
    #[test]
    fn heap_alloc_works() {
        let layout = core::alloc::Layout::from_size_align(256, 8).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        assert!(!ptr.is_null(), "heap allocation should succeed");
        let slice = unsafe { core::slice::from_raw_parts(ptr, 256) };
        assert!(slice.iter().all(|&b| b == 0), "alloc_zeroed should be zero");
        unsafe { alloc::alloc::dealloc(ptr, layout) };
    }

    /// Allocate a page-aligned 4KB block, write and read back a value.
    #[test]
    fn large_heap_alloc() {
        let layout = core::alloc::Layout::from_size_align(4096, 4096).unwrap();
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        assert!(!ptr.is_null(), "4KB heap allocation should succeed");
        unsafe {
            core::ptr::write_volatile(ptr, 0xAA);
            assert_eq!(core::ptr::read_volatile(ptr), 0xAA);
        }
        unsafe { alloc::alloc::dealloc(ptr, layout) };
    }
}

pub use tests::suite;
