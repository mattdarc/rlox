use super::linemap::LineMap;
use super::memory::{AllocError, Block, BlockPtr};
use super::policy::AllocationPolicy;

/// Bump-allocated block containing lines. Objects can be allocated in unused lines
pub struct BumpBlock {
    cursor: usize,

    /// The limit for immix is either the next occupied line, or the end of the block
    limit: usize,
    mem: Block,
    used_lines: LineMap,
}

impl BumpBlock {
    pub fn new<A: AllocationPolicy>() -> Result<Self, AllocError> {
        Ok(BumpBlock {
            cursor: 0,
            limit: A::LINES_PER_BLOCK,
            mem: Block::new(A::BLOCK_SIZE_BYTES)?,
            used_lines: LineMap::new(A::LINES_PER_BLOCK),
        })
    }

    /// Try to alloc memory of the requested size in this block, starting at the cursor. If the
    /// space cannot be allocated, `None` is returned
    pub fn inner_alloc(&mut self, bytes: usize) -> Option<BlockPtr> {
        None
    }
}
