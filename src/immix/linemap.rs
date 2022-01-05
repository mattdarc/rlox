use bit_vec::BitVec;

/// Type that marks used lines within a block
pub struct LineMap(BitVec);

impl LineMap {
    pub fn new(size: usize) -> Self {
        LineMap(BitVec::from_elem(size, false))
    }

    pub fn is_used(&self, line: usize) -> bool {
        self.0[line]
    }

    pub fn set_used(&mut self, line: usize) {
        self.0.set(line, true)
    }

    pub fn set_unused(&mut self, line: usize) {
        self.0.set(line, false);
    }

    pub fn set_range_used(&mut self, start: usize, end: usize) {
        assert!(
            self.0.iter().skip(start).take(end - start).all(|x| !x),
            "Set already used line as used!"
        );

        for line in start..end {
            self.set_used(line);
        }
    }

    pub fn set_range_unused(&mut self, start: usize, end: usize) {
        assert!(
            self.0.iter().skip(start).take(end - start).all(|x| x),
            "Set already unused line as unused!"
        );

        for line in start..end {
            self.set_unused(line);
        }
    }

    pub fn entire_block_used(&self) -> bool {
        self.0.all()
    }

    pub fn find_next_used(&self, line: usize) -> usize {
        assert!(
            !self.is_used(line),
            "Finding the next used line should be called from an unused line!"
        );
        line + self.0.iter().skip(line).take_while(|x| !x).count()
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
    fn next_used() {
        let mut map = LineMap::new(128);
        for i in (0..128).step_by(10) {
            map.set_used(i);
        }

        let next_used_line = map.find_next_used(1);
        assert_eq!(next_used_line, 10);

        let next_used_line = map.find_next_used(11);
        assert_eq!(next_used_line, 20);
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
