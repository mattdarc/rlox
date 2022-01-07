/// Defines the allocation strategy of the Immix allocator/garbage collector
pub trait AllocationPolicy {
    const BLOCK_SIZE_BYTES: usize;
    const LINE_SIZE_BYTES: usize;
    const LINES_PER_BLOCK: usize = Self::BLOCK_SIZE_BYTES / Self::LINE_SIZE_BYTES;
}

/// Defines the reclamation strategy of the Immix allocator/garbage collector
pub trait ReclamationPolicy {}
