use ngvm::operations::{UOp, CheckedAdd, BiOp};

fn main() {
    let x_invoked = BiOp::<CheckedAdd>::invoke(&10, &10);
    println!("Executing {:?}", x_invoked)
}
