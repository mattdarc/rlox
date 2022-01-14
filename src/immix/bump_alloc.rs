use super::linemap::LineMap;
use super::memory::{AllocError, Block};
use super::policy::AllocationPolicy;
use std::ptr::NonNull;

/// Each block can be in one of 3 states:
///
///   `Free`       : Completely unused
///   `Recyclable` : Partially unused (at least F free lines)
///   `Unavailable`: Completely used
///
/// The Immix paper suggests to use F = 1.
pub enum BlockState {
    Free,
    Recyclable,
    Unavailable,
}

/// Pointer to a GC-managed block of memory. Copy is implemented here because the GC will handle
/// ownership.
///
/// TODO: A better idea *might* be to have the size kept separately and have just `inner` be copied
/// around as that is a single 8-byte value instead of 2
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ManagedPtr {
    inner: NonNull<u8>,
    size: usize,
}

impl ManagedPtr {
    fn new(inner: NonNull<u8>, size: usize) -> ManagedPtr {
        ManagedPtr { inner, size }
    }
}

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

    /// Mark the bytes pointed to by the `ptr` as unused, allowing them to be re-used by
    /// `inner_alloc`
    pub fn inner_dealloc(&mut self, ptr: ManagedPtr) {
        self.used_lines.set_range_unused(
            ptr.inner.as_ptr() as usize - self.mem.as_ptr() as usize,
            ptr.size,
        );
    }

    /// Try to alloc memory of the requested size in this block, starting at the cursor. If the
    /// space cannot be allocated, `None` is returned
    pub fn inner_alloc(&mut self, bytes: usize) -> Option<ManagedPtr> {
        if self.cursor == self.limit {
            if let Some((hole_begin, hole_end_exclusive)) = self.find_first_hole() {
                self.cursor = hole_begin;
                self.limit = hole_end_exclusive;
            } else {
                return None;
            }
        }

        assert!(
            self.cursor < self.limit,
            "The cursor must be less than or equal to the limit"
        );

        let next_used = self.used_lines.find_next_used(self.cursor);
        let num_lines_available = next_used - self.cursor;

        if num_lines_available >= bytes {
            // Allocate the bytes for this block, updating the cursor and limit accordingly. If the
            // cursor is greater than the limit, they will be updated lazily on request for new
            // memory
            let block_start = self.cursor;
            let block_end_exclusive = block_start + bytes;

            self.used_lines
                .set_range_used(block_start, block_end_exclusive);
            self.cursor += bytes;

            // This operation is safe because we *know* mem is NonNull
            return Some(ManagedPtr::new(
                unsafe { NonNull::new_unchecked(self.mem.as_ptr().wrapping_add(block_start)) },
                bytes,
            ));
        }

        None
    }

    /// Return the current state of the block -- see `BlockState`.
    pub fn get_block_state(&self) -> BlockState {
        if self.used_lines.entire_block_used() {
            return BlockState::Unavailable;
        } else if self.used_lines.entire_block_unused() {
            return BlockState::Free;
        }

        return BlockState::Recyclable;
    }

    /// Returns `true` if this block is the one that allocated the `ManagedPtr`, false otherwise.
    pub fn contains(&self, ptr: &ManagedPtr) -> bool {
        let block_start = self.mem.as_ptr() as usize;
        let block_end = block_start + self.mem.size();
        let ptr_addr = ptr.inner.as_ptr() as usize;

        ptr_addr >= block_start && ptr_addr < block_end
    }

    /// Return the first hole (group of unused lines) in the block starting at the first line.
    /// Returns `None` if no such hole exists
    fn find_first_hole(&self) -> Option<(usize, usize)> {
        if self.used_lines.entire_block_used() {
            return None;
        }

        let hole_begin = self.used_lines.find_next_unused(0);
        assert!(
            hole_begin != self.used_lines.len(),
            "The entire block is not used. There should be an unused line!"
        );

        let hole_end_exclusive = self.used_lines.find_next_used(hole_begin);
        Some((hole_begin, hole_end_exclusive))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestAllocator;
    impl AllocationPolicy for TestAllocator {
        const BLOCK_SIZE_BYTES: usize = 256;
        const LINE_SIZE_BYTES: usize = 64;
    }

    fn is_range_unused(block: &BumpBlock, start: usize, end: usize) -> bool {
        (start..end)
            .map(|i| block.used_lines.is_used(i))
            .all(|x| !x)
    }

    #[test]
    fn allocate_bytes() {
        let mut bump_block = BumpBlock::new::<TestAllocator>().expect("Could not allocate block!");
        assert_eq!(bump_block.cursor, 0);
        assert_eq!(bump_block.limit, 4);
        assert!(is_range_unused(&bump_block, 0, 4));

        let single_line_ptr = bump_block.inner_alloc(1).expect("Did not allocate line!");
        assert_eq!(single_line_ptr.inner.as_ptr(), bump_block.mem.as_ptr());
        assert_eq!(single_line_ptr.size, 1);

        assert_eq!(bump_block.cursor, 1);
        assert_eq!(bump_block.limit, 4);

        let double_line_ptr = bump_block.inner_alloc(2).expect("Did not allocate line!");
        assert_eq!(
            double_line_ptr.inner.as_ptr(),
            bump_block.mem.as_ptr().wrapping_add(1)
        );
        assert_eq!(double_line_ptr.size, 2);

        assert_eq!(bump_block.cursor, 3);
        assert_eq!(bump_block.limit, 4);

        // No slots are available for another double-line ptr
        assert_eq!(bump_block.inner_alloc(2), None);
        assert_eq!(bump_block.cursor, 3);
        assert_eq!(bump_block.limit, 4);
    }

    #[test]
    fn dealloc_bytes() {
        let mut bump_block = BumpBlock::new::<TestAllocator>().expect("Could not allocate block!");
        let ptr1 = bump_block
            .inner_alloc(2)
            .expect("Could not allocate first ptr!");
        let _ptr2 = bump_block
            .inner_alloc(2)
            .expect("Could not allocate second ptr!");

        assert_eq!(bump_block.cursor, 4);
        assert_eq!(bump_block.limit, 4);

        // de-allocation should not change the internal state other than the unused lines
        bump_block.inner_dealloc(ptr1);
        assert!(is_range_unused(&bump_block, 0, 2));
        assert_eq!(bump_block.cursor, 4);
        assert_eq!(bump_block.limit, 4);

        // Try to re-allocate a new smaller region. The cursor and limit should reflect a new hole
        let _small_ptr = bump_block
            .inner_alloc(1)
            .expect("Could not allocate small ptr!");
        assert_eq!(bump_block.cursor, 1);
        assert_eq!(bump_block.limit, 2);
    }

    #[test]
    fn block_contains_ptr() {
        let mut bump_block = BumpBlock::new::<TestAllocator>().expect("Could not allocate block!");
        let ptr = bump_block.inner_alloc(2).expect("Could not allocate ptr!");
        assert!(bump_block.contains(&ptr));

        let other_bump_block =
            BumpBlock::new::<TestAllocator>().expect("Could not allocate block!");
        assert!(!other_bump_block.contains(&ptr));
    }
}
