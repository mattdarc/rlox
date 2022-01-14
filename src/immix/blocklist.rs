use super::bump_alloc::{BumpBlock, ManagedPtr};
use super::memory::AllocError;
use super::policy::AllocationPolicy;

/// List of `BumpBlock`s that have been allocated, in address-order.
pub struct BlockList {
    blocks: Vec<BumpBlock>,
}

impl BlockList {
    pub fn new() -> Self {
        BlockList { blocks: Vec::new() }
    }

    pub fn alloc<A: AllocationPolicy>(&mut self, bytes: usize) -> Result<ManagedPtr, AllocError> {
        for block in self.blocks.iter_mut() {
            if let Some(ptr) = block.inner_alloc(bytes) {
                return Ok(ptr);
            }
        }

        self.blocks.push(BumpBlock::new::<A>()?);
        let new_block = self.blocks.last_mut().unwrap();

        Ok(new_block.inner_alloc(bytes).expect(&format!(
            "Object too large to allocate in {:?}",
            A::BLOCK_SIZE_BYTES
        )))
    }

    pub fn dealloc(&mut self, ptr: ManagedPtr) {}
}
