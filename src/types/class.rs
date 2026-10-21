//! Classes of primitive types, each class is an enum of its member types, so a type narrowed to a
//! class can be matched exhaustively

use super::PrimitiveType;

/// A set of primitive types, narrowed from a [`PrimitiveType`] by the type checker
pub trait PrimitiveClass: Copy + Into<PrimitiveType> {
    /// Name of the class in errors
    const NAME: &'static str;

    /// The type as a member of the class, `None` if it is not one
    fn narrow(t: PrimitiveType) -> Option<Self>;
}

macro_rules! primitive_classes {
    ($($(#[$attr:meta])* $name:ident: $desc:literal { $($variant:ident),+ $(,)? })*) => {
        $(
        $(#[$attr])*
        #[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl From<$name> for PrimitiveType {
            fn from(obj: $name) -> Self {
                match obj {
                    $($name::$variant => PrimitiveType::$variant),+
                }
            }
        }

        impl PrimitiveClass for $name {
            const NAME: &'static str = $desc;

            fn narrow(t: PrimitiveType) -> Option<Self> {
                match t {
                    $(PrimitiveType::$variant => Some($name::$variant),)+
                    _ => None,
                }
            }
        }
        )*
    };
}

primitive_classes! {
    IntegerType: "integer" { U64, U32, U16, U8, I64, I32, I16, I8 }
    UnsignedType: "unsigned integer" { U64, U32, U16, U8 }
    SignedType: "signed integer" { I64, I32, I16, I8 }
    FloatType: "float" { F64, F32 }
    NumberType: "number" { U64, U32, U16, U8, I64, I32, I16, I8, F64, F32 }
    IntegerOrBoolType: "integer or bool" { U64, U32, U16, U8, I64, I32, I16, I8, Bool }
    SignedOrFloatType: "signed integer or float" { I64, I32, I16, I8, F64, F32 }
}
