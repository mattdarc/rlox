use std::ptr::NonNull;

#[derive(Debug, PartialEq)]
pub enum AllocError {
    OutOfMemory,
    BadAlignment,
}

#[derive(Debug)]
pub struct Block {
    ptr: BlockPtr,
    size: BlockSize,
}

pub type BlockPtr = NonNull<u8>;
pub type BlockSize = usize;
pub type BlockResult = Result<Block, AllocError>;

impl Block {
    pub fn new(size: BlockSize) -> BlockResult {
        Ok(internal::alloc_block(size)?)
    }

    pub fn into_ptr_mut(self) -> BlockPtr {
        self.ptr
    }

    pub fn size(&self) -> BlockSize {
        self.size
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        internal::dealloc_block(self)
    }
}

mod internal {
    use super::*;
    use std::alloc::{alloc, dealloc, Layout};

    pub fn alloc_block(size: BlockSize) -> BlockResult {
        if !size.is_power_of_two() {
            return Err(AllocError::BadAlignment);
        }

        let ptr = unsafe { alloc(Layout::from_size_align_unchecked(size, size)) };

        if let Some(ptr) = NonNull::new(ptr) {
            return Ok(Block { ptr, size });
        }

        return Err(AllocError::OutOfMemory);
    }

    pub fn dealloc_block(block: &mut Block) {
        let size = block.size;
        unsafe {
            dealloc(
                block.ptr.as_ptr(),
                Layout::from_size_align_unchecked(size, size),
            )
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn alloc_dealloc(size: BlockSize) -> Result<(), AllocError> {
        let block = Block::new(size)?;

        // The block address bitwise AND the alignment bits (size - 1) should
        // be a mutually exclusive set of bits
        let mask = size - 1;
        assert!((block.ptr.as_ptr() as usize & mask) ^ mask == mask);
        // ANCHOR_END: TestAllocPointer

        drop(block);
        Ok(())
    }

    #[test]
    fn test_bad_sizealign() {
        assert!(alloc_dealloc(999) == Err(AllocError::BadAlignment))
    }

    #[test]
    fn test_4k() {
        assert!(alloc_dealloc(4096).is_ok())
    }

    #[test]
    fn test_32k() {
        assert!(alloc_dealloc(32768).is_ok())
    }

    #[test]
    fn test_16m() {
        assert!(alloc_dealloc(16 * 1024 * 1024).is_ok())
    }
}
