use ngvm::operations::{markers, BiOp};

fn main() {
    let x_invoked = BiOp::<markers::Eq>::invoke(&10, &10);
    println!("Executing {:?}", x_invoked)
}
