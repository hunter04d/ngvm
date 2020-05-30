use ngvm::vm::refs::LocatedRef;

fn main() {
    println!("size of located_ref: {}", std::mem::size_of::<LocatedRef>());
    println!("Executing...");
}
