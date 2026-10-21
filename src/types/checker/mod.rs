use std::borrow::Cow;
use std::fmt;

use checked::Checked;
pub use operands::{OperandTypesChecker, PrimitiveOperandsChecker};
use pointed::{RefTypeChecker, SArrTypeChecker};
use primitive::PrimitiveTaggedType;
use primitive::PrimitiveTypeChecker;

use crate::error::VmError;
use crate::types::{RefKind, VmType};

mod checked;
mod operands;
mod pointed;
mod primitive;
pub mod tags;

#[derive(Debug, Default)]
pub struct TypeCheckerCtx {
    errors: Vec<TypeError>,
}

pub trait TypeCheckerCtxProvider {
    fn type_checker_ctx() -> TypeCheckerCtx;
}

pub fn combine_checks<T1, T2>(
    r1: Result<T1, Vec<TypeError>>,
    r2: Result<T2, Vec<TypeError>>,
) -> Result<(T1, T2), Vec<TypeError>> {
    match (r1, r2) {
        (Ok(r1), Ok(r2)) => Ok((r1, r2)),
        (Err(e1), Ok(_)) => Err(e1),
        (Ok(_), Err(e2)) => Err(e2),
        (Err(mut e1), Err(e2)) => Err({
            e1.extend(e2);
            e1
        }),
    }
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
            Err(std::mem::take(errors))
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

/// Names a checked type in errors
#[derive(Debug, Clone)]
pub enum Tag {
    Name(Cow<'static, str>),
    /// Operands checked together, named by their tags
    Group(&'static [&'static str]),
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Name(name) => f.write_str(name),
            Tag::Group(tags) => f.write_str(&tags.join(", ")),
        }
    }
}

impl From<&'static str> for Tag {
    fn from(name: &'static str) -> Self {
        Tag::Name(Cow::Borrowed(name))
    }
}

impl From<String> for Tag {
    fn from(name: String) -> Self {
        Tag::Name(Cow::Owned(name))
    }
}

impl From<Cow<'static, str>> for Tag {
    fn from(name: Cow<'static, str>) -> Self {
        Tag::Name(name)
    }
}

#[derive(Debug)]
pub struct TaggedType {
    pub tag: Tag,
    pub vm_type: VmType,
}

pub struct TypeChecker<'a, C: HasTypeCheckerCtx> {
    tag: Tag,
    vm_type: Checked<&'a VmType>,
    ctx: C,
}

impl TaggedType {
    pub fn new(tag: impl Into<Tag>, vm_type: &VmType) -> Self {
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
        self.tag("")
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
    AllNotEqual(Vec<TaggedType>),
    NotReference(TaggedType),
    NotMutReference(TaggedType),
    NotArray(TaggedType),
}

#[derive(Debug, Copy, Clone)]
pub enum RefCondition {
    Ref,
    Mut,
    Any,
}

impl RefCondition {
    fn get_error(&self, t: TaggedType) -> TypeError {
        match self {
            RefCondition::Ref => TypeError::NotReference(t),
            RefCondition::Mut => TypeError::NotMutReference(t),
            RefCondition::Any => TypeError::NotReference(t),
        }
    }

    fn satisfies(&self, ref_kind: RefKind) -> bool {
        match self {
            RefCondition::Ref => matches!(ref_kind, RefKind::Ref),
            RefCondition::Mut => matches!(ref_kind, RefKind::Mut),
            RefCondition::Any => true,
        }
    }
}

impl<'a, C: HasTypeCheckerCtx> TypeChecker<'a, C> {
    pub fn new(tag: impl Into<Tag>, vm_type: &'a VmType, ctx: C) -> Self {
        Self {
            tag: tag.into(),
            vm_type: Checked::known(vm_type),
            ctx,
        }
    }

    pub fn primitive(mut self) -> PrimitiveTypeChecker<C> {
        let t = self.vm_type.and_then(|t| match t.primitive() {
            Some(p) => Checked::known(p),
            None => {
                let err = TypeError::NotPrimitive(t.tag(self.tag.clone()));
                Checked::poisoned(&mut self.ctx, err)
            }
        });
        PrimitiveTypeChecker {
            tag: self.tag,
            t,
            ctx: self.ctx,
        }
    }

    pub fn of_ref(mut self, cond: RefCondition) -> RefTypeChecker<'a, C> {
        let ref_type = self.vm_type.and_then(|t| {
            let err = || cond.get_error(t.tag(self.tag.clone()));
            match t.ref_type() {
                Some(r) => {
                    if !cond.satisfies(r.kind) {
                        self.ctx.report(err());
                    }
                    Checked::known(r)
                }
                None => Checked::poisoned(&mut self.ctx, err()),
            }
        });
        RefTypeChecker {
            ref_type,
            tag: self.tag,
            ctx: self.ctx,
        }
    }

    /// Checks that the type is some kind of reference
    pub fn any_ref(self) -> RefTypeChecker<'a, C> {
        self.of_ref(RefCondition::Any)
    }

    /// Checks that the type is a *Mut* reference
    pub fn mut_ref(self) -> RefTypeChecker<'a, C> {
        self.of_ref(RefCondition::Mut)
    }

    /// Checks that the type is a *Ref* reference
    pub fn ref_ref(self) -> RefTypeChecker<'a, C> {
        self.of_ref(RefCondition::Ref)
    }

    pub fn s_arr(mut self) -> SArrTypeChecker<'a, C> {
        let arr_type = self.vm_type.and_then(|t| match t.s_arr() {
            Some(arr) => Checked::known(arr),
            None => {
                let err = TypeError::NotArray(t.tag(self.tag.clone()));
                Checked::poisoned(&mut self.ctx, err)
            }
        });
        SArrTypeChecker {
            arr_type,
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
