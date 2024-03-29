use bit_vec::BitVec;

/// Type that marks used lines within a block
pub struct LineMap(BitVec);

impl LineMap {
    pub fn new(size: usize) -> Self {
        LineMap(BitVec::from_elem(size, false))
    }

    /// Returns true if the line `line` is used
    pub fn is_used(&self, line: usize) -> bool {
        self.0[line]
    }

    /// Set the line `line` as used
    pub fn set_used(&mut self, line: usize) {
        self.0.set(line, true)
    }

    /// Set the line `line` as unused
    pub fn set_unused(&mut self, line: usize) {
        self.0.set(line, false);
    }

    /// Set each line in the range from start..end as used
    pub fn set_range_used(&mut self, start: usize, end: usize) {
        assert!(
            self.0.iter().skip(start).take(end - start).all(|x| !x),
            "Set already used line as used!"
        );

        for line in start..end {
            self.set_used(line);
        }
    }

    /// Set each line in the range from start..end as unused
    pub fn set_range_unused(&mut self, start: usize, end: usize) {
        assert!(
            self.0.iter().skip(start).take(end - start).all(|x| x),
            "Set already unused line as unused!"
        );

        for line in start..end {
            self.set_unused(line);
        }
    }

    /// Returns true if the whole block is unused
    pub fn entire_block_unused(&self) -> bool {
        self.0.none()
    }

    /// Returns true if the whole block is used
    pub fn entire_block_used(&self) -> bool {
        self.0.all()
    }

    /// Returns the next used line, or 1 past the end if no lines are used
    pub fn find_next_used(&self, line: usize) -> usize {
        line + self.0.iter().skip(line).take_while(|x| !x).count()
    }

    /// Returns the next unused line, or 1 past the end if no lines are used
    pub fn find_next_unused(&self, line: usize) -> usize {
        line + self.0.iter().skip(line).take_while(|x| *x).count()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn print(&self) -> String {
        self.0
            .iter()
            .map(|used| if used { "1" } else { "0" })
            .fold(String::new(), |r, s| r + s)
            .to_owned()
    }

    pub fn dump(&self) {
        println!("{}", self.print());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn used_unused() {
        let mut map = LineMap::new(128);
        assert!(!map.entire_block_used());

        map.set_used(10);
        assert!(map.is_used(10));

        map.set_unused(10);
        assert!(!map.is_used(10));
    }

    #[test]
    fn next_unused() {
        let mut map = LineMap::new(128);
        for i in (0..128).step_by(10) {
            map.set_used(i);
        }

        let next_unused_line = map.find_next_unused(0);
        assert_eq!(next_unused_line, 1);

        map.set_used(127);
        map.set_used(126);
        let next_unused_line = map.find_next_unused(126);
        assert_eq!(next_unused_line, 128);
    }

    #[test]
    fn next_used() {
        let mut map = LineMap::new(128);
        for i in (0..128).step_by(10) {
            map.set_used(i);
        }

        let next_used_line = map.find_next_used(1);
        assert_eq!(next_used_line, 10);

        let next_used_line = map.find_next_used(11);
        assert_eq!(next_used_line, 20);

        let next_used_line = map.find_next_used(125);
        assert_eq!(next_used_line, 128);
    }

    #[test]
    fn entire_block_unused() {
        let mut map = LineMap::new(10);
        map.set_range_used(0, 10);
        assert!(!map.entire_block_unused());

        map.set_range_unused(0, 3);
        assert!(!map.entire_block_unused());

        map.set_range_unused(3, 10);
        assert!(map.entire_block_unused());
    }

    #[test]
    fn entire_block_used() {
        let mut map = LineMap::new(10);
        map.set_range_used(0, 10);
        assert!(map.entire_block_used());

        map.set_range_unused(0, 3);
        assert!(!map.entire_block_used());
    }
}
