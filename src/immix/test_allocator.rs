use crate::immix::policy::AllocationPolicy;

pub struct TestAllocator;
impl AllocationPolicy for TestAllocator {
    const BLOCK_SIZE_BYTES: usize = 256;
    const LINE_SIZE_BYTES: usize = 64;
}
