use crate::bytecode::{BcInstr, Chunk, Register, Value};
use std::cell::RefCell;
use std::mem::MaybeUninit;

const STACK_MAX: usize = 256;
const REGISTER_MAX: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileErr,
    RuntimeErr,
}

pub struct VM {
    stack: RefCell<[Value; STACK_MAX]>,
    chunk: Chunk,
    ip: usize,
}

impl VM {
    pub fn with_chunk(chunk: Chunk) -> Self {
        VM {
            stack: RefCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
            chunk,
            ip: 0,
        }
    }

    pub fn new() -> Self {
        VM::with_chunk(Chunk::new())
    }

    pub fn load_program(&mut self, chunk: Chunk) {
        self.chunk = chunk;
        self.ip = 0;
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.load_program(chunk);
        self.run()
    }

    fn load(&self, r: Register) -> Value {
        self.stack.borrow()[r.num()]
    }

    fn store(&self, dest: Register, v: Value) {
        self.stack.borrow_mut()[dest.num()] = v;
    }

    fn step(&mut self) -> Option<InterpretResult> {
        let ip = self.ip;
        self.ip += 1;

        #[cfg(debug_assertions)]
        {
            println!();
            for r in 0..REGISTER_MAX {
                println!("[{}]", self.stack.borrow()[r]);
            }
            println!("{}", self.chunk.dump_instr(ip));
        }

        macro_rules! binary_op {
            ($op: tt, $dest:ident, $a:ident, $b:ident) => {
                {
                    let a = self.load($a);
                    let b = self.load($b);
                    self.store($dest, a $op b);
                }
            };
        }

        match &self.chunk.instrs()[ip] {
            &BcInstr::Ret => return Some(InterpretResult::Ok),
            &BcInstr::Add { dest, a, b } => binary_op!(+, dest, a, b),
            &BcInstr::Sub { dest, a, b } => binary_op!(-, dest, a, b),
            &BcInstr::Mul { dest, a, b } => binary_op!(*, dest, a, b),
            &BcInstr::Div { dest, a, b } => binary_op!(/, dest, a, b),
            &BcInstr::Neg { dest, a } => self.store(dest, -self.load(a)),
            &BcInstr::LoadConst { dest, id } => self.store(dest, self.chunk.constant(id)),
        }

        None
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            match self.step() {
                Some(ir) => return ir,
                None => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn negate_value() {
        let mut program = Chunk::new();

        let ret = Register::ret();

        let id = program.add_constant(10.11);
        program.write(BcInstr::LoadConst { dest: ret, id }, 0);
        program.write(BcInstr::Neg { dest: ret, a: ret }, 0);
        program.write(BcInstr::Neg { dest: ret, a: ret }, 0);
        program.write(BcInstr::Ret, 1);

        let mut vm = VM::new();
        vm.load_program(program);

        // LoadConst
        let result = vm.step();
        assert_eq!(result, None);
        assert_eq!(vm.load(ret), 10.11);

        let result = vm.step();
        // Neg
        assert_eq!(result, None);
        assert_eq!(vm.load(ret), -10.11);

        // Neg
        let result = vm.step();
        assert_eq!(result, None);
        assert_eq!(vm.load(ret), 10.11);

        // Ret
        let result = vm.step();
        assert_eq!(result, Some(InterpretResult::Ok));
        assert_eq!(vm.load(ret), 10.11);
    }

    /*
    #[test]
    fn expression_tests() {
        /// Macro to construct VM state from a post-fix expression:
        macro_rules! vm_state {
            ($c:ident, - $($rest:tt)*) => {
                $c.write(BcInstr::Sub, 0);
                vm_state!($c, $($rest)*);
            };

            ($c:ident, + $($rest:tt)*) => {
                $c.write(BcInstr::Add, 0);
                vm_state!($c, $($rest)*);
            };

            ($c:ident, * $($rest:tt)*) => {
                $c.write(BcInstr::Mul, 0);
                vm_state!($c, $($rest)*);
            };

            ($c:ident, / $($rest:tt)*) => {
                $c.write(BcInstr::Div, 0);
                vm_state!($c, $($rest)*);
            };

            ($c:ident, $a:tt $($rest:tt)*) => {
                $c.add_constant($a as Value, 0);
                vm_state!($c, $($rest)*);
            };

            ($c:ident,) => {};
        }

        macro_rules! make_expression_test {
            ([$($rpn:tt)*] == $expected:literal) => {
                {
                    let mut chunk = Chunk::new();
                    vm_state!(chunk, $($rpn)*);
                    chunk.write(BcInstr::Ret, 0);

                    let mut vm = VM::with_chunk(chunk);
                    vm.run();
                    assert_eq!(vm.load(Register::ret()), $expected as Value);
                }
            };
        }

        make_expression_test!([1 2 3 4 + - /] == -0.2);
        make_expression_test!([1 2 + 4 -] == -1);
        make_expression_test!([400 2 100 * / (-4) -] == 6);
    }
    */
}
