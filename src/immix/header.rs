/// Required information for all heap-allocated objects
pub trait ObjectHeader {
    fn make_header(&self) -> Box<dyn ObjectHeader>;
}
