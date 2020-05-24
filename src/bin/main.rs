use ngvm::{ConstantPool, Module, Vm};

fn main() {
    let _ = Vm::with_module(Module::new(ConstantPool::new(Vec::new())));
    println!("Executing...");
}
