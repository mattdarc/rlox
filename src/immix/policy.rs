use std::marker::PhantomData;
use std::ptr::NonNull;

/// Defines the allocation strategy of the Immix allocator/garbage collector
pub trait AllocationPolicy {
    const BLOCK_SIZE_BYTES: usize;
    const LINE_SIZE_BYTES: usize;
    const LINES_PER_BLOCK: usize = Self::BLOCK_SIZE_BYTES / Self::LINE_SIZE_BYTES;
}

/// Defines the reclamation strategy of the Immix allocator/garbage collector
pub trait ReclamationPolicy {}

pub struct ImmixGc<A: AllocationPolicy, R: ReclamationPolicy> {
    allocation_policy: PhantomData<A>,
    reclamation_policy: PhantomData<R>,
}

pub struct DefaultAllocation;
pub struct DefaultReclamation;

impl ReclamationPolicy for DefaultReclamation {}
impl AllocationPolicy for DefaultAllocation {
    const BLOCK_SIZE_BYTES: usize = 32 * 1024;
    const LINE_SIZE_BYTES: usize = 128;
}

/// Default implementation of Immix
pub type StickyImmix = ImmixGc<DefaultAllocation, DefaultReclamation>;

impl<A: AllocationPolicy, R: ReclamationPolicy> ImmixGc<A, R> {
    /// Allocate the object of type `T`, returning the pointer to the object. Checks space in the
    /// bump allocator in the following order:
    ///
    ///  Look for open lines in address order in a recycled block
    ///  Repeat (1) in the next recycled block
    ///  Request a new block from the global allocator
    pub fn alloc<T>(&mut self, object: T) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(std::ptr::null_mut()) }
    }
}
