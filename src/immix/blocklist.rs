use super::bump_alloc::{BumpBlock, ManagedPtr};
use super::memory::AllocError;
use super::policy::AllocationPolicy;

/// List of `BumpBlock`s that have been allocated, in address-order.
pub struct BlockList<A: AllocationPolicy> {
    blocks: Vec<BumpBlock<A>>,
}

impl<A: AllocationPolicy> BlockList<A> {
    pub fn new() -> Self {
        BlockList { blocks: Vec::new() }
    }

    /// Allocate a block of size `bytes` from the BlockList. Will allocate from the first block
    /// that fits
    pub fn alloc(&mut self, bytes: usize) -> Result<ManagedPtr, AllocError> {
        for block in self.blocks.iter_mut() {
            if let Some(ptr) = block.inner_alloc(bytes) {
                return Ok(ptr);
            }
        }

        self.blocks.push(BumpBlock::<A>::new()?);
        let new_block = self.blocks.last_mut().unwrap();

        Ok(new_block.inner_alloc(bytes).expect(&format!(
            "Object too large to allocate in {:?}",
            A::BLOCK_SIZE_BYTES
        )))
    }

    pub fn dealloc(&mut self, ptr: ManagedPtr) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::immix::test_allocator::TestAllocator;

    #[test]
    fn alloc_dealloc_blocks() {
        let mut blist = BlockList::<TestAllocator>::new();

        let mut ptrs = Vec::new();
        for _ in 0..10 {
            ptrs.push(blist.alloc(64).expect("Could not allocate block!"));
        }
        // We should have 3 blocks in our list
        assert_eq!(blist.blocks.len(), 3);

        for i in 0..ptrs.len() {
            assert!(blist.blocks[i / 4].contains(&ptrs[i]));
        }
    }
}
