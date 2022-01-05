use librlox::*;

fn main() {
    let mut instrs = bytecode::Chunk::new();
    let id = instrs.add_constant(1.2);
    instrs.write(
        bytecode::BcInstr::LoadConst {
            dest: bytecode::Register::ret(),
            id,
        },
        123,
    );
    instrs.write(bytecode::BcInstr::Ret, 123);

    let mut vm = vm::VM::new();
    vm.interpret(instrs);
}
