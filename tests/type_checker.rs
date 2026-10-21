#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne};

use ngvm::types::checker::{
    HasTypeCheckerCtx, OperandTypesChecker, TypeChecker, TypeCheckerCtx, TypeError, tags,
};
use ngvm::types::{
    PointedType, PrimitiveType, RefLocation, ThreePrimitiveTypes, TwoPrimitiveTypes, VmType,
};

fn root_checker(t: &VmType) -> TypeChecker<'_, TypeCheckerCtx> {
    TypeChecker::new("test", t, Default::default())
}

#[test]
fn test_ref_to_array_of_primitive() {
    let arr = PointedType::s_arr(PrimitiveType::U64, 10);
    let rf = PointedType::mut_reference(arr, RefLocation::Stack);
    let t = VmType::from(rf);
    let ch = root_checker(&t);
    let res = ch
        .mut_ref()
        .to()
        .s_arr()
        .of_type()
        .primitive()
        .equals(PrimitiveType::U64)
        .and()
        .get();

    assert!(res.is_ok());
}

#[test]
fn test_refs_deep() {
    let rf1 = PointedType::ref_reference(PrimitiveType::U64, RefLocation::Stack);
    let rf2 = PointedType::ref_reference(rf1, RefLocation::Stack);
    let rf3 = PointedType::ref_reference(rf2, RefLocation::Stack);
    let t = VmType::from(rf3);
    let ch = root_checker(&t);
    let res = ch
        .any_ref()
        .to()
        .any_ref()
        .to()
        .any_ref()
        .to()
        .primitive()
        .equals(PrimitiveType::U64)
        .and()
        .get();
    assert!(res.is_ok());
}

#[test]
fn test_array_is_not_a_ref() {
    let t = VmType::from(PointedType::s_arr(PrimitiveType::U64, 3));
    let res = root_checker(&t).any_ref().to().s_arr().get();
    let errors = res.expect_err("an array is not a reference");
    assert!(
        matches!(errors[..], [TypeError::NotReference(_)]),
        "{errors:?}"
    );
}

#[test]
fn test_ref_to_ref_is_not_an_array() {
    let inner = PointedType::ref_reference(PrimitiveType::U64, RefLocation::Stack);
    let t = VmType::from(PointedType::ref_reference(inner, RefLocation::Stack));
    let res = root_checker(&t).any_ref().to().s_arr().get();
    let errors = res.expect_err("a reference is not an array");
    assert!(matches!(errors[..], [TypeError::NotArray(_)]), "{errors:?}");
}

#[test]
fn test_primitive_is_not_an_array() {
    let t = VmType::from(PrimitiveType::U64);
    let errors = root_checker(&t)
        .s_arr()
        .get()
        .expect_err("u64 is not an array");
    assert!(matches!(errors[..], [TypeError::NotArray(_)]), "{errors:?}");
}

fn integer_or_bool(t: &VmType) -> Result<(), Vec<TypeError>> {
    root_checker(t)
        .primitive()
        .either()
        .integer()
        .or()
        .bool()
        .and()
        .fmt(|_, t| format!("{:?} is neither integer or bool", t.vm_type))
        .get()
}

#[test]
fn test_either_passes_on_any_alternative() {
    assert!(integer_or_bool(&PrimitiveType::U32.into()).is_ok());
    assert!(integer_or_bool(&PrimitiveType::Bool.into()).is_ok());
}

#[test]
fn test_either_fails_when_no_alternative_matches() {
    let errors = integer_or_bool(&PrimitiveType::F64.into()).expect_err("f64 is neither");
    assert!(matches!(errors[..], [TypeError::From(..)]), "{errors:?}");
}

#[test]
fn test_poisoned_type_reports_only_the_first_error() {
    let t = VmType::from(PrimitiveType::U64);
    let errors = root_checker(&t)
        .any_ref()
        .to()
        .s_arr()
        .of_type()
        .primitive()
        .equals(PrimitiveType::U64)
        .and()
        .get()
        .expect_err("u64 is not a reference");
    assert!(
        matches!(errors[..], [TypeError::NotReference(_)]),
        "{errors:?}"
    );
}

fn tags_of(error: &TypeError) -> Vec<String> {
    match error {
        TypeError::AllNotEqual(types) => types.iter().map(|t| t.tag.to_string()).collect(),
        TypeError::NotPrimitive(t) | TypeError::Condition(t, _) => vec![t.tag.to_string()],
        e => panic!("unexpected error {e:?}"),
    }
}

#[test]
fn test_all_same_reports_every_operand() {
    let (u32_t, u64_t) = (PrimitiveType::U32.into(), PrimitiveType::U64.into());
    let mut ctx = TypeCheckerCtx::new();
    let errors = OperandTypesChecker::three(&u32_t, &u64_t, &u32_t, &mut ctx)
        .all_primitives()
        .all_same()
        .get()
        .expect_err("u32 and u64 differ");
    assert_eq!(errors.len(), 1);
    assert_eq!(tags_of(&errors[0]), ["result", "op1", "op2"]);
}

#[test]
fn test_same_checks_only_its_group() {
    let (u64_t, u8_t) = (PrimitiveType::U64.into(), PrimitiveType::U8.into());
    let mut ctx = TypeCheckerCtx::new();
    let types = OperandTypesChecker::three(&u64_t, &u64_t, &u8_t, &mut ctx)
        .all_primitives()
        .op2()
        .one_of(&[PrimitiveType::U32, PrimitiveType::U16, PrimitiveType::U8])
        .and()
        .same(&[tags::RESULT, tags::OP1])
        .and()
        .get()
        .expect("op2 is not in the group");
    assert_eq!(
        types,
        ThreePrimitiveTypes {
            result: PrimitiveType::U64,
            op1: PrimitiveType::U64,
            op2: PrimitiveType::U8,
        }
    );
}

#[test]
fn test_two_operands_keep_their_types() {
    let (bool_t, u32_t) = (PrimitiveType::Bool.into(), PrimitiveType::U32.into());
    let mut ctx = TypeCheckerCtx::new();
    let types = OperandTypesChecker::two(&bool_t, &u32_t, &mut ctx)
        .all_primitives()
        .result()
        .bool()
        .and()
        .op()
        .integer()
        .and()
        .get()
        .expect("bool result and integer operand");
    assert_eq!(
        types,
        TwoPrimitiveTypes {
            result: PrimitiveType::Bool,
            op: PrimitiveType::U32,
        }
    );
}

#[test]
fn test_non_primitive_operand_is_reported_once() {
    let u64_t = PrimitiveType::U64.into();
    let arr = VmType::from(PointedType::s_arr(PrimitiveType::U64, 3));
    let mut ctx = TypeCheckerCtx::new();
    let errors = OperandTypesChecker::three(&u64_t, &arr, &u64_t, &mut ctx)
        .all_primitives()
        .all_same()
        .integer()
        .and()
        .get()
        .expect_err("op1 is an array");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(matches!(errors[0], TypeError::NotPrimitive(_)));
    assert_eq!(tags_of(&errors[0]), ["op1"]);
}

#[test]
fn test_group_type_is_checked_under_the_group_tag() {
    let (bool_t, f64_t) = (PrimitiveType::Bool.into(), PrimitiveType::F64.into());
    let mut ctx = TypeCheckerCtx::new();
    let errors = OperandTypesChecker::three(&bool_t, &f64_t, &f64_t, &mut ctx)
        .all_primitives()
        .result()
        .bool()
        .and()
        .same_operands()
        .integer()
        .and()
        .get()
        .expect_err("f64 is not an integer");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(tags_of(&errors[0]), ["op1, op2"]);
}
