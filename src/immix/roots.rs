use super::bump_alloc::ManagedPtr;

/// Stores the pointers to the objects allocated in the block list. These pointers are searched
/// transitively to find the lines in each block that are not used. When a used line is found it is
/// marked as such in the line map. After tracing is complete, unused blocks are returned to the
/// block list for allocation (right now we don't need to do this step since we don't have separate
/// used/unused lists).
struct ApplicationRoots {
    roots: Vec<ManagedPtr>,
}
