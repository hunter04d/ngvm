//! Functions implemented in Rust

use std::fmt::{self, Debug, Formatter};
use std::ops::Range;

use crate::code::refs::StackRef;
use crate::error::VmError;
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::checker::{Taggable, TypeError};
use crate::types::{HasPrimitiveType, PointedType, PrimitiveType, RefKind, VmType};

use super::{static_str, Access, Result, Vm};

type NativeBody = dyn Fn(&mut NativeCall<'_>) -> Result<()>;

/// Body of a native [`Function`](crate::Function)
pub struct NativeFn(Box<NativeBody>);

impl NativeFn {
    pub fn new(f: impl Fn(&mut NativeCall<'_>) -> Result<()> + 'static) -> Self {
        Self(Box::new(f))
    }

    pub(crate) fn call(&self, call: &mut NativeCall<'_>) -> Result<()> {
        (self.0)(call)
    }
}

impl Debug for NativeFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("NativeFn")
    }
}

/// A call of a native function, its only access to the vm
///
/// The function reads its arguments, which are the values of its stack frame, and pushes the
/// values it returns. When it is done, the pushed values are returned like with
/// [`RetN`](crate::opcodes::Opcode::RetN): they are checked against the signature,
/// and a reference cannot be returned unless it was passed as an argument.
pub struct NativeCall<'vm> {
    vm: &'vm mut Vm,
    args: usize,
}

impl<'vm> NativeCall<'vm> {
    pub(super) fn new(vm: &'vm mut Vm, args: usize) -> Self {
        Self { vm, args }
    }

    pub fn arg_count(&self) -> usize {
        self.args
    }

    pub fn arg_type(&self, index: usize) -> Result<&VmType> {
        let arg = self.arg_ref(index)?;
        Ok(&self.vm.stack_metadata(arg)?.value_type)
    }

    /// Value of the argument `index`
    pub fn arg<T: NativeValue>(&self, index: usize) -> Result<T> {
        let data = self.arg_data(index, T::get_type())?;
        Ok(T::from_data(*data.first().ok_or(VmError::BadVmState)?))
    }

    /// Static string of the argument `index`
    pub fn arg_str(&self, index: usize) -> Result<&str> {
        static_str(self.arg_data(index, PrimitiveType::SStr)?)
    }

    /// Value the reference argument `index` points to
    pub fn read<T: NativeValue>(&self, index: usize) -> Result<T> {
        let range = self.pointee(index, T::get_type(), RefKind::Ref)?;
        let data = self.vm.stack.get(range.start).ok_or(VmError::BadVmState)?;
        Ok(T::from_data(*data))
    }

    /// Static string the reference argument `index` points to
    pub fn read_str(&self, index: usize) -> Result<&str> {
        let range = self.pointee(index, PrimitiveType::SStr, RefKind::Ref)?;
        static_str(self.vm.stack.get(range).ok_or(VmError::BadVmState)?)
    }

    /// Writes `value` to the value the mutable reference argument `index` points to
    pub fn write<T: NativeValue>(&mut self, index: usize, value: T) -> Result<()> {
        let range = self.pointee(index, T::get_type(), RefKind::Mut)?;
        let data = self.vm.stack.get_mut(range.start).ok_or(VmError::BadVmState)?;
        *data = value.into_data();
        Ok(())
    }

    /// Returns `value` after the values returned so far
    pub fn ret<T: NativeValue>(&mut self, value: T) {
        self.vm.push_single_typed(value.into_data(), T::get_type());
    }

    /// Returns the static string `value` after the values returned so far
    ///
    /// A string of the vm, e.g. from [`arg_str`](Self::arg_str), is returned with
    /// [`ret_arg`](Self::ret_arg)
    pub fn ret_str(&mut self, value: &'static str) {
        self.vm.push_s_str(value.as_ptr() as usize, value.len());
    }

    /// Returns the argument `index` after the values returned so far,
    /// moving it out unless it is `Copy`
    pub fn ret_arg(&mut self, index: usize) -> Result<()> {
        let arg = self.arg_ref(index)?;
        self.vm.push_moved(arg)
    }

    fn arg_ref(&self, index: usize) -> Result<StackRef> {
        if index < self.args {
            Ok(StackRef(index))
        } else {
            Err(VmError::NoSuchArgument {
                index,
                count: self.args,
            })
        }
    }

    /// Stack data of the argument `index`, which must be of type `t`
    fn arg_data(&self, index: usize, t: PrimitiveType) -> Result<&[StackData]> {
        let arg = self.arg_ref(index)?;
        let meta = self.vm.checked_stack_metadata(arg, Access::Read)?;
        let expected = VmType::from(t);
        if meta.value_type != expected {
            return Err(arg_type_error(&meta.value_type, index, expected));
        }
        self.vm.stack_data(arg)
    }

    /// Stack data range of the value of type `t` that the reference argument `index` points to
    ///
    /// A shared reference is enough for `RefKind::Ref`, a mutable one is needed for `RefKind::Mut`
    fn pointee(&self, index: usize, t: PrimitiveType, kind: RefKind) -> Result<Range<usize>> {
        let arg = self.arg_ref(index)?;
        let meta = self.vm.checked_stack_metadata(arg, Access::Read)?;
        let pointed = VmType::from(t);
        let matches = meta
            .value_type
            .ref_type()
            .is_some_and(|r| r.pointer == pointed && (kind == RefKind::Ref || r.kind == kind));
        if !matches {
            let expected = PointedType::reference(pointed, kind).into();
            return Err(arg_type_error(&meta.value_type, index, expected));
        }
        let (_, range) = self.vm.pointee(self.vm.absolute_index(arg))?;
        Ok(range)
    }
}

fn arg_type_error(actual: &VmType, index: usize, expected: VmType) -> VmError {
    let actual = actual.tag(format!("arg{index}"));
    VmError::TypeError(vec![TypeError::NotEquals(actual, expected)])
}

/// A primitive value that a native function can read and return
///
/// Implemented for the integers, floats, `bool`, `char` and `()`
pub trait NativeValue: HasPrimitiveType + sealed::Sealed {}

mod sealed {
    use super::StackData;

    pub trait Sealed: Sized {
        fn from_data(data: StackData) -> Self;
        fn into_data(self) -> StackData;
    }
}

macro_rules! impl_native_value {
    ($($t: ty),*) => {
        $(
            impl NativeValue for $t {}

            impl sealed::Sealed for $t {
                fn from_data(data: StackData) -> Self {
                    <$t>::from_single(data)
                }

                fn into_data(self) -> StackData {
                    self.into_stack_data()
                }
            }
        )*
    };
}

impl_native_value!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, bool, char);

impl NativeValue for () {}

impl sealed::Sealed for () {
    fn from_data(_: StackData) -> Self {}

    fn into_data(self) -> StackData {
        self.into_stack_data()
    }
}
