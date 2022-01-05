/// Represents all values in rlox
pub type Value = f64;

/// ID of a constant. Used as index into the constant data section
pub type ConstantId = u16;

/// Register in the VM, represented as a `u8`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Register(u8);

impl Register {
    pub fn new(r: u8) -> Self {
        Register(r)
    }

    pub fn num(&self) -> usize {
        self.0 as usize
    }

    pub fn ret() -> Self {
        Register(0)
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "%r{}", self.0)
    }
}

/// Bytecode instruction for rlox VM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcInstr {
    Ret,
    Neg {
        dest: Register,
        a: Register,
    },
    Add {
        dest: Register,
        a: Register,
        b: Register,
    },
    Sub {
        dest: Register,
        a: Register,
        b: Register,
    },
    Mul {
        dest: Register,
        a: Register,
        b: Register,
    },
    Div {
        dest: Register,
        a: Register,
        b: Register,
    },
    LoadConst {
        dest: Register,
        id: ConstantId,
    },
}

/// Representation of line numbers using an RLE encoding
#[derive(Debug)]
struct RLELine {
    line: usize,
    count: usize,
}

pub struct Chunk {
    code: Vec<BcInstr>,
    lines: Vec<RLELine>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, v: Value) -> ConstantId {
        (if let Some((i, _)) = self.constants.iter().enumerate().find(|(_, &c)| c == v) {
            i
        } else {
            self.constants.push(v);
            self.constants.len() - 1
        }) as ConstantId
    }

    pub fn write(&mut self, instr: BcInstr, line: usize) {
        self.code.push(instr);
        if let Some(rle_line) = self.lines.last_mut() {
            if rle_line.line == line {
                rle_line.count += 1;
                return;
            }
        }

        self.lines.push(RLELine { line, count: 1 });
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.lines.clear();
    }

    pub fn get_line(&self, instr_index: usize) -> usize {
        let mut line_count = 0;
        for RLELine { line, count } in &self.lines {
            line_count += count;

            if line_count > instr_index {
                return *line;
            }
        }
        panic!("Inconsistency in source lines! {:?}", self.lines);
    }

    pub fn instrs(&self) -> &[BcInstr] {
        &self.code
    }

    pub fn constant(&self, index: ConstantId) -> Value {
        self.constants[index as usize]
    }

    #[cfg(debug_assertions)]
    pub fn dump_instr(&self, offset: usize) -> String {
        let s = match &self.code[offset] {
            BcInstr::Ret => format!("RET {}", Register::ret()),
            BcInstr::LoadConst { dest, id } => format!("LOAD {} <= {}", dest, self.constant(*id)),
            BcInstr::Neg { dest, a } => format!("NEG {} <= {}", dest, a),
            BcInstr::Add { dest, a, b } => format!("ADD {} <= {}, {}", dest, a, b),
            BcInstr::Sub { dest, a, b } => format!("SUB {} <= {}, {}", dest, a, b),
            BcInstr::Mul { dest, a, b } => format!("MUL {} <= {}, {}", dest, a, b),
            BcInstr::Div { dest, a, b } => format!("DIV {} <= {}, {}", dest, a, b),
        };

        format!("0x{:X} {}", offset, s)
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "== CHUNK ==\n")?;
        for (offset, _instr) in self.code.iter().enumerate() {
            f.write_fmt(format_args!("0x{:X} {} ", offset, self.get_line(offset)))?;
            f.write_str(&self.dump_instr(offset))?;
        }

        writeln!(f, "\n-- DATA  --\n")?;
        for (offset, constant) in self.constants.iter().enumerate() {
            f.write_fmt(format_args!("0x{:X} ", offset))?;
            f.write_str(&constant.to_string())?;
            writeln!(f, "\n")?;
        }
        writeln!(f, "=== END ===")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size_of_instr() {
        let size = std::mem::size_of::<BcInstr>();
        assert!(size <= 4, "BcInstr is size {}", size);
    }

    #[test]
    fn get_instr_line() {
        let mut instrs = Chunk::new();
        instrs.write(BcInstr::Ret, 0);
        instrs.write(BcInstr::Ret, 0);
        instrs.write(BcInstr::Ret, 0);
        instrs.write(BcInstr::Ret, 1);
        instrs.write(BcInstr::Ret, 2);
        instrs.write(BcInstr::Ret, 3);

        assert_eq!(instrs.get_line(0), 0);
        assert_eq!(instrs.get_line(1), 0);
        assert_eq!(instrs.get_line(2), 0);
        assert_eq!(instrs.get_line(3), 1);
        assert_eq!(instrs.get_line(4), 2);
        assert_eq!(instrs.get_line(5), 3);
    }
}
