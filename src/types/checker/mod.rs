use crate::error::VmError;
use crate::types::VmType;
use smallvec::alloc::borrow::Cow;

mod primitive;
use crate::types::checker::primitive::TwoPrimitiveTypesChecker;
pub use primitive::ThreePrimitiveTypesChecker;

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

    fn ctx(&mut self) -> &mut TypeCheckerCtx;

    fn unwrap(self) -> Self::Unwrapped;

    fn get(mut self) -> Result<Self::Unwrapped, Vec<TypeError>> {
        if self.ctx().errors.is_empty() {
            Ok(HasTypeCheckerCtx::unwrap(self))
        } else {
            let errors = &mut self.ctx().errors;
            Err(errors.drain(..).collect())
        }
    }
    fn get_vm(self) -> Result<Self::Unwrapped, VmError> {
        self.get().map_err(VmError::TypeError)
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

    fn ctx(&mut self) -> &mut TypeCheckerCtx {
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

pub trait Taggable {
    fn tag(&self, tag: impl Into<Tag>) -> TaggedType;

    fn no_tag(&self) -> TaggedType {
        self.tag(Tag::Borrowed(""))
    }
}

impl Taggable for VmType {
    fn tag(&self, tag: impl Into<Tag>) -> TaggedType {
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
    TwoNotEqual(TaggedType, TaggedType),
    ThreeNotEqual(TaggedType, TaggedType, TaggedType),
    AllNotEqual(Vec<TaggedType>),
}

impl<'a, 'c> ThreeTypesChecker<'a, 'c> {
    pub fn all_primitives(self) -> ThreePrimitiveTypesChecker<'c> {
        let result = self.result.primitive();
        let op1 = self.op1.primitive();
        let op2 = self.op2.primitive();
        if result.is_none() {
            self.ctx
                .report(TypeError::NotPrimitive(self.result.tag(tags::RESULT)));
        }
        if op1.is_none() {
            self.ctx
                .report(TypeError::NotPrimitive(self.op1.tag(tags::OP1)));
        }
        if op2.is_none() {
            self.ctx
                .report(TypeError::NotPrimitive(self.op2.tag(tags::OP1)));
        }
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
