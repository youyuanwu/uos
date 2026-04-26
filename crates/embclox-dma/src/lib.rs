#![no_std]

/// A region of DMA-coherent memory.
///
/// This is a plain data handle (vaddr, paddr, size) — not an owner.
/// Ownership is managed by whoever called `DmaAllocator::alloc_coherent`.
pub struct DmaRegion {
    pub vaddr: usize,
    pub paddr: usize,
    pub size: usize,
}

/// Allocator for DMA-coherent memory regions.
///
/// Uses `&self` — implementations use interior mutability if needed.
/// `alloc_coherent` panics on failure (boot-time, fixed system).
///
/// # Ownership model
///
/// `alloc_coherent` returns a `DmaRegion` handle. The caller is
/// responsible for calling `free_coherent` when the region is no longer
/// needed. Device drivers do this automatically in their `Drop` impl,
/// so users of the driver don't need to manage DMA lifetimes manually.
///
/// This mirrors Linux's `dma_alloc_coherent`/`dma_free_coherent` pattern.
/// The alloc call itself is not `unsafe` — allocating memory doesn't
/// violate memory safety. The unsafe part is using the returned addresses
/// for hardware DMA (telling the device to read/write at paddr).
pub trait DmaAllocator {
    /// Allocate DMA-coherent memory with the given size and alignment.
    /// Panics on failure.
    fn alloc_coherent(&self, size: usize, align: usize) -> DmaRegion;

    /// Free a previously allocated DMA region.
    ///
    /// # Safety
    /// The caller must ensure no references to the region's memory exist.
    /// Using the region's vaddr or paddr after this call is undefined behavior.
    unsafe fn free_coherent(&self, region: &DmaRegion);
}
