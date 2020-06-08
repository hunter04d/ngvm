use smallvec::alloc::borrow::Cow;

pub use primitive::ThreePrimitiveTypesChecker;

use crate::error::VmError;
use crate::types::{PointedType, RefKind, VmType};
use pointed::{RefTypeChecker, SArrTypeChecker};
use primitive::PrimitiveTaggedType;
use primitive::{PrimitiveTypeChecker, TwoPrimitiveTypesChecker};

mod pointed;
mod primitive;
pub mod tags;
#[derive(Debug)]
pub struct TypeCheckerCtx {
    errors: Vec<TypeError>,
}

pub trait TypeCheckerCtxProvider {
    fn type_checker_ctx() -> TypeCheckerCtx;
}

pub trait HasTypeCheckerCtx: Sized {
    type Unwrapped;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx;

    fn unwrap(self) -> Self::Unwrapped;

    fn get(mut self) -> Result<Self::Unwrapped, Vec<TypeError>> {
        if self.root_ctx().errors.is_empty() {
            Ok(HasTypeCheckerCtx::unwrap(self))
        } else {
            let errors = &mut self.root_ctx().errors;
            Err(errors.drain(..).collect())
        }
    }
    fn get_vm(self) -> Result<Self::Unwrapped, VmError> {
        self.get().map_err(VmError::TypeError)
    }

    fn report(&mut self, e: TypeError) {
        self.root_ctx().report(e);
    }
}

impl TypeCheckerCtx {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn report(&mut self, e: TypeError) {
        self.errors.push(e);
    }
}

impl TypeCheckerCtxProvider for TypeCheckerCtx {
    fn type_checker_ctx() -> TypeCheckerCtx {
        Self::new()
    }
}
impl Default for TypeCheckerCtx {
    fn default() -> Self {
        Self { errors: Vec::new() }
    }
}

impl HasTypeCheckerCtx for TypeCheckerCtx {
    type Unwrapped = ();

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self
    }

    fn unwrap(self) -> Self::Unwrapped {}

    fn report(&mut self, e: TypeError) {
        TypeCheckerCtx::report(self, e)
    }
}

impl HasTypeCheckerCtx for &mut TypeCheckerCtx {
    type Unwrapped = ();

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self
    }

    fn unwrap(self) -> Self::Unwrapped {}
}

pub type Tag = Cow<'static, str>;

#[derive(Debug)]
pub struct TaggedType {
    pub tag: Tag,
    pub vm_type: VmType,
}

pub struct TypeChecker<'a, C: HasTypeCheckerCtx> {
    pub tag: Tag,
    pub vm_type: Option<&'a VmType>,
    pub ctx: C,
}

pub struct ThreeTypesChecker<'a, 'c> {
    pub result: &'a VmType,
    pub op1: &'a VmType,
    pub op2: &'a VmType,
    pub ctx: &'c mut TypeCheckerCtx,
}

pub struct TwoTypesChecker<'a, 'c> {
    pub result: &'a VmType,
    pub op: &'a VmType,
    pub ctx: &'c mut TypeCheckerCtx,
}

impl TaggedType {
    pub fn new(tag: impl Into<Cow<'static, str>>, vm_type: &VmType) -> Self {
        Self {
            tag: tag.into(),
            vm_type: vm_type.clone(),
        }
    }

    #[allow(dead_code)]
    fn with_empty_tag(vm_type: &VmType) -> Self {
        Self {
            tag: "".into(),
            vm_type: vm_type.clone(),
        }
    }
}

impl From<PrimitiveTaggedType> for TaggedType {
    fn from(obj: PrimitiveTaggedType) -> Self {
        Self {
            tag: obj.tag,
            vm_type: obj.t.into(),
        }
    }
}

pub trait Taggable {
    type Tagged;

    fn tag(&self, tag: impl Into<Tag>) -> Self::Tagged;

    fn no_tag(&self) -> Self::Tagged {
        self.tag(Cow::Borrowed(""))
    }
}

impl Taggable for VmType {
    type Tagged = TaggedType;
    fn tag(&self, tag: impl Into<Tag>) -> Self::Tagged {
        TaggedType {
            tag: tag.into(),
            vm_type: self.clone(),
        }
    }
}

#[derive(Debug)]
pub enum TypeError {
    NotPrimitive(TaggedType),
    NotEquals(TaggedType, VmType),
    NotOneOf(TaggedType, Vec<VmType>),
    Condition(TaggedType, String),
    From(Box<TypeError>, String),
    TwoNotEqual(TaggedType, TaggedType),
    ThreeNotEqual(TaggedType, TaggedType, TaggedType),
    AllNotEqual(Vec<TaggedType>),
    NotReference(TaggedType),
    NotMutReference(TaggedType),
}

impl<'a, C: HasTypeCheckerCtx> TypeChecker<'a, C> {
    pub fn primitive(mut self) -> PrimitiveTypeChecker<C> {
        let p = match self.vm_type {
            None => None,
            Some(&VmType::Primitive(p)) => Some(p),
            Some(t) => {
                self.ctx
                    .report(TypeError::NotPrimitive(t.tag(self.tag.clone())));
                None
            }
        };
        PrimitiveTypeChecker {
            tag: self.tag,
            t: p,
            ctx: self.ctx,
        }
    }

    pub fn any_ref(mut self) -> RefTypeChecker<'a, C> {
        let r = match self.vm_type {
            None => None,
            Some(VmType::PointedType(bpt)) => {
                if let PointedType::Ref(r) = bpt.as_ref() {
                    Some(r)
                } else {
                    None
                }
            }
            Some(t) => {
                self.ctx
                    .report(TypeError::NotReference(t.tag(self.tag.clone())));
                None
            }
        };
        RefTypeChecker {
            ref_type: r,
            tag: self.tag,
            ctx: self.ctx,
        }
    }

    pub fn mut_ref(mut self) -> RefTypeChecker<'a, C> {
        let r = match self.vm_type {
            None => None,
            Some(VmType::PointedType(bpt)) => {
                if let PointedType::Ref(r) = bpt.as_ref() {
                    if matches!(r.kind, RefKind::Mut) {
                        Some(r)
                    } else {
                        let t = VmType::PointedType(bpt.clone());
                        self.report(TypeError::NotMutReference(t.tag(self.tag.clone())));
                        None
                    }
                } else {
                    None
                }
            }
            Some(t) => {
                self.report(TypeError::NotMutReference(t.tag(self.tag.clone())));
                None
            }
        };
        RefTypeChecker {
            ref_type: r,
            tag: self.tag,
            ctx: self.ctx,
        }
    }

    pub fn s_arr(mut self) -> SArrTypeChecker<'a, C> {
        let arr = match self.vm_type {
            None => None,
            Some(VmType::PointedType(bpt)) => {
                if let PointedType::SArr(arr) = bpt.as_ref() {
                    Some(arr)
                } else {
                    None
                }
            }
            Some(t) => {
                self.ctx
                    .report(TypeError::NotPrimitive(t.tag(self.tag.clone())));
                None
            }
        };
        SArrTypeChecker {
            arr_type: arr,
            tag: self.tag,
            ctx: self.ctx,
        }
    }
}

impl<'a, C: HasTypeCheckerCtx> HasTypeCheckerCtx for TypeChecker<'a, C> {
    type Unwrapped = &'a VmType;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        self.vm_type.unwrap()
    }
}

impl<'a, 'c> ThreeTypesChecker<'a, 'c> {
    pub fn all_primitives(self) -> ThreePrimitiveTypesChecker<'c> {
        let result = if let Some(p) = self.result.primitive() {
            Some(p)
        } else {
            self.ctx
                .report(TypeError::NotPrimitive(self.result.tag(tags::RESULT)));
            None
        };
        let op1 = if let Some(p) = self.op1.primitive() {
            Some(p)
        } else {
            self.ctx
                .report(TypeError::NotPrimitive(self.op1.tag(tags::OP1)));
            None
        };
        let op2 = if let Some(p) = self.op2.primitive() {
            Some(p)
        } else {
            self.ctx
                .report(TypeError::NotPrimitive(self.op2.tag(tags::OP1)));
            None
        };
        ThreePrimitiveTypesChecker {
            result,
            op1,
            op2,
            ctx: self.ctx,
        }
    }
}

impl<'a, 'c> TwoTypesChecker<'a, 'c> {
    pub fn all_primitives(self) -> TwoPrimitiveTypesChecker<'c> {
        let result = self.result.primitive();
        let op = self.op.primitive();
        if result.is_none() {
            self.ctx
                .report(TypeError::NotPrimitive(self.result.tag(tags::RESULT)));
        }
        if op.is_none() {
            self.ctx
                .report(TypeError::NotPrimitive(self.op.tag(tags::OP)));
        }
        TwoPrimitiveTypesChecker {
            result,
            op,
            ctx: self.ctx,
        }
    }
}
