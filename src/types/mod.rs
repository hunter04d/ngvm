pub use pointed::*;
pub use primitive::*;

pub mod checker;
mod pointed;
mod primitive;

#[derive(Debug, PartialEq, Clone)]
pub enum VmType {
    Primitive(PrimitiveType),
    PointedType(Box<PointedType>),
}

impl VmType {
    pub fn is_primitive(&self) -> bool {
        matches!(self, VmType::Primitive(_))
    }

    pub fn primitive(&self) -> Option<PrimitiveType> {
        if let VmType::Primitive(t) = self {
            Some(*t)
        } else {
            None
        }
    }

    pub fn pointed(&self) -> Option<&PointedType> {
        if let VmType::PointedType(b) = self {
            Some(b.as_ref())
        } else {
            None
        }
    }

    pub fn s_arr(&self) -> Option<&SArrType> {
        if let PointedType::SArr(arr) = self.pointed()? {
            Some(arr)
        } else {
            None
        }
    }

    pub fn ref_type(&self) -> Option<&RefType> {
        if let PointedType::Ref(rf) = self.pointed()? {
            Some(rf)
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        match self {
            VmType::Primitive(p) => p.size(),
            VmType::PointedType(p) => p.size(),
        }
    }

    pub fn is_copy(&self) -> bool {
        match self {
            VmType::Primitive(_) => true,
            VmType::PointedType(p) => match p.as_ref() {
                PointedType::SArr(a) => a.pointer.is_copy(),
                PointedType::Ref(r) => r.is_copy(),
            },
        }
    }
}

pub trait HasVmType {
    fn vm_type(&self) -> &VmType;
}

impl HasVmType for VmType {
    fn vm_type(&self) -> &VmType {
        self
    }
}

impl From<PrimitiveType> for VmType {
    fn from(obj: PrimitiveType) -> Self {
        VmType::Primitive(obj)
    }
}

impl From<PointedType> for VmType {
    fn from(obj: PointedType) -> Self {
        VmType::PointedType(Box::new(obj))
    }
}

impl From<RefType> for VmType {
    fn from(obj: RefType) -> Self {
        VmType::PointedType(Box::new(PointedType::Ref(obj)))
    }
}

impl From<SArrType> for VmType {
    fn from(obj: SArrType) -> Self {
        VmType::PointedType(Box::new(PointedType::SArr(obj)))
    }
}

#[derive(Debug, PartialEq)]
pub struct ThreePrimitiveTypes {
    pub result: PrimitiveType,
    pub op1: PrimitiveType,
    pub op2: PrimitiveType,
}

#[derive(Debug, PartialEq)]
pub struct TwoPrimitiveTypes {
    pub result: PrimitiveType,
    pub op: PrimitiveType,
}
