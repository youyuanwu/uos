extern crate alloc;

use embclox_hal_x86::memory::MemoryMapper;

/// Global memory mapper for tests that need MMIO mapping.
static mut MAPPER: Option<MemoryMapper> = None;

/// # Safety
/// Must be called once from single-threaded init before `suite()`.
pub unsafe fn init(phys_offset: u64, kernel_offset: u64) {
    unsafe {
        *core::ptr::addr_of_mut!(MAPPER) = Some(MemoryMapper::new(phys_offset, kernel_offset));
    }
}

fn mapper() -> &'static mut MemoryMapper {
    unsafe {
        (*core::ptr::addr_of_mut!(MAPPER))
            .as_mut()
            .expect("memory mapper not initialized")
    }
}

/// Heap and MMIO mapping tests. Heap is global after HAL init.
/// MMIO tests use map/unmap to avoid page table conflicts.
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

    /// Map an MMIO region, verify the virtual address, then unmap.
    #[test]
    fn map_and_unmap_mmio() {
        let m = mapper();
        let mapping = m.map_mmio(0xFED0_0000, 0x1000);
        assert!(
            mapping.vaddr() >= 0x4000_0000_0000,
            "MMIO vaddr should be in upper region, got {:#x}",
            mapping.vaddr()
        );
        // Safety: no references to the mapped region exist after this point.
        unsafe { m.unmap_mmio(&mapping) };
    }

    /// Verify map_mmio returns increasing virtual addresses.
    #[test]
    fn map_mmio_advances_cursor() {
        let m = mapper();
        let m1 = m.map_mmio(0xFED0_0000, 0x1000);
        let m2 = m.map_mmio(0xFED0_1000, 0x1000);
        assert!(
            m2.vaddr() > m1.vaddr(),
            "second map should return higher address: v1={:#x}, v2={:#x}",
            m1.vaddr(),
            m2.vaddr()
        );
        // Safety: no references to the mapped regions exist after this point.
        unsafe {
            m.unmap_mmio(&m1);
            m.unmap_mmio(&m2);
        }
    }
}

pub use tests::suite;
