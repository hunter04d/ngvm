/// 3 reference opcode
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ThreeRefs {
    pub result: usize,
    pub op1: usize,
    pub op2: usize,
}

/// 3 reference constructor for opcodes
pub fn three(result: usize, op1: usize, op2: usize) -> ThreeRefs {
    ThreeRefs { result, op1, op2 }
}
