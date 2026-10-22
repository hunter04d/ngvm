#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne};

use ngvm::types::checker::{
    HasTypeCheckerCtx, ThreeOperandsChecker, TwoOperandsChecker, TypeChecker, TypeCheckerCtx,
    TypeError,
};
use ngvm::types::{
    FloatType, IntegerOrBoolType, IntegerType, NumberType, PointedType, PrimitiveType,
    SignedType, ThreePrimitiveTypes, TwoPrimitiveTypes, VmType,
};

fn root_checker(t: &VmType) -> TypeChecker<'_, TypeCheckerCtx> {
    TypeChecker::new("test", t, Default::default())
}

#[test]
fn test_ref_to_array_of_primitive() {
    let arr = PointedType::s_arr(PrimitiveType::U64, 10);
    let rf = PointedType::mut_reference(arr);
    let t = VmType::from(rf);
    let ch = root_checker(&t);
    let res = ch
        .mut_ref()
        .to(|t| {
            t.s_arr()
                .of_type(|t| t.primitive().equals(PrimitiveType::U64))
        })
        .get();

    assert!(res.is_ok());
}

#[test]
fn test_refs_deep() {
    let rf1 = PointedType::ref_reference(PrimitiveType::U64);
    let rf2 = PointedType::ref_reference(rf1);
    let rf3 = PointedType::ref_reference(rf2);
    let t = VmType::from(rf3);
    let ch = root_checker(&t);
    let res = ch
        .any_ref()
        .to(|t| {
            t.any_ref()
                .to(|t| t.any_ref().to(|t| t.primitive().equals(PrimitiveType::U64)))
        })
        .get();
    assert!(res.is_ok());
}

#[test]
fn test_array_is_not_a_ref() {
    let t = VmType::from(PointedType::s_arr(PrimitiveType::U64, 3));
    let res = root_checker(&t).any_ref().to(|t| t.s_arr()).get();
    let errors = res.expect_err("an array is not a reference");
    assert!(
        matches!(errors[..], [TypeError::NotReference(_)]),
        "{errors:?}"
    );
}

#[test]
fn test_ref_to_ref_is_not_an_array() {
    let inner = PointedType::ref_reference(PrimitiveType::U64);
    let t = VmType::from(PointedType::ref_reference(inner));
    let res = root_checker(&t).any_ref().to(|t| t.s_arr()).get();
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

fn integer_or_bool(t: &VmType) -> Result<IntegerOrBoolType, Vec<TypeError>> {
    root_checker(t).primitive().is::<IntegerOrBoolType>().get()
}

#[test]
fn test_class_narrows_the_type() {
    assert_eq!(
        integer_or_bool(&PrimitiveType::U32.into()).unwrap(),
        IntegerOrBoolType::U32
    );
    assert_eq!(
        integer_or_bool(&PrimitiveType::Bool.into()).unwrap(),
        IntegerOrBoolType::Bool
    );
}

#[test]
fn test_type_outside_the_class_is_reported() {
    let errors = integer_or_bool(&PrimitiveType::F64.into()).expect_err("f64 is neither");
    assert!(
        matches!(errors[..], [TypeError::NotOfClass(_, "integer or bool")]),
        "{errors:?}"
    );
}

#[test]
fn test_type_outside_the_class_poisons_later_checks() {
    let t = VmType::from(PrimitiveType::F64);
    let errors = root_checker(&t)
        .primitive()
        .integer()
        .unsigned()
        .get()
        .expect_err("f64 is not an integer");
    assert!(
        matches!(errors[..], [TypeError::NotOfClass(_, "integer")]),
        "{errors:?}"
    );
}

#[test]
fn test_class_can_be_narrowed_further() {
    let t = VmType::from(PrimitiveType::I16);
    let res = root_checker(&t).primitive().number().signed().get();
    assert_eq!(res.unwrap(), SignedType::I16);
}

#[test]
fn test_poisoned_type_reports_only_the_first_error() {
    let t = VmType::from(PrimitiveType::U64);
    let errors = root_checker(&t)
        .any_ref()
        .to(|t| {
            t.s_arr()
                .of_type(|t| t.primitive().equals(PrimitiveType::U64))
        })
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
        TypeError::NotPrimitive(t) | TypeError::Condition(t, _) | TypeError::NotOfClass(t, _) => {
            vec![t.tag.to_string()]
        }
        e => panic!("unexpected error {e:?}"),
    }
}

#[test]
fn test_all_same_reports_every_operand() {
    let (u32_t, u64_t) = (PrimitiveType::U32.into(), PrimitiveType::U64.into());
    let mut ctx = TypeCheckerCtx::new();
    let errors = ThreeOperandsChecker::new(&u32_t, &u64_t, &u32_t, &mut ctx)
        .all_same()
        .get()
        .expect_err("u32 and u64 differ");
    assert_eq!(errors.len(), 1);
    assert_eq!(tags_of(&errors[0]), ["result", "op1", "op2"]);
}

#[test]
fn test_same_result_and_op1_checks_only_them() {
    let (u64_t, u8_t) = (PrimitiveType::U64.into(), PrimitiveType::U8.into());
    let mut ctx = TypeCheckerCtx::new();
    let types = ThreeOperandsChecker::new(&u64_t, &u64_t, &u8_t, &mut ctx)
        .same_result_and_op1()
        .result(|t| t.integer())
        .op(|t| t.one_of(&[PrimitiveType::U32, PrimitiveType::U16, PrimitiveType::U8]))
        .get()
        .expect("op2 is not in the group");
    assert_eq!(
        types,
        TwoPrimitiveTypes {
            result: IntegerType::U64,
            op: PrimitiveType::U8,
        }
    );
}

#[test]
fn test_operands_keep_their_narrowed_types() {
    let (bool_t, u32_t) = (PrimitiveType::Bool.into(), PrimitiveType::U32.into());
    let mut ctx = TypeCheckerCtx::new();
    let types = TwoOperandsChecker::new(&bool_t, &u32_t, &mut ctx)
        .result(|t| t.bool())
        .op(|t| t.integer())
        .get()
        .expect("bool result and integer operand");
    assert_eq!(
        types,
        TwoPrimitiveTypes {
            result: PrimitiveType::Bool,
            op: IntegerType::U32,
        }
    );
}

#[test]
fn test_non_primitive_operand_is_reported_once() {
    let u64_t = PrimitiveType::U64.into();
    let arr = VmType::from(PointedType::s_arr(PrimitiveType::U64, 3));
    let mut ctx = TypeCheckerCtx::new();
    let errors = ThreeOperandsChecker::new(&u64_t, &arr, &u64_t, &mut ctx)
        .all_same()
        .integer()
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
    let errors = ThreeOperandsChecker::new(&bool_t, &f64_t, &f64_t, &mut ctx)
        .same_operands()
        .result(|t| t.bool())
        .op(|t| t.integer())
        .get()
        .expect_err("f64 is not an integer");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(tags_of(&errors[0]), ["op1, op2"]);
}

#[test]
fn test_same_operands_continues_with_two_operands() {
    let (bool_t, f32_t) = (PrimitiveType::Bool.into(), PrimitiveType::F32.into());
    let mut ctx = TypeCheckerCtx::new();
    let types = ThreeOperandsChecker::new(&bool_t, &f32_t, &f32_t, &mut ctx)
        .same_operands()
        .result(|t| t.bool())
        .op(|t| t.number())
        .get()
        .expect("bool result and number operands");
    assert_eq!(
        types,
        TwoPrimitiveTypes {
            result: PrimitiveType::Bool,
            op: NumberType::F32,
        }
    );
}

#[test]
fn test_merged_operands_are_named_by_their_operands() {
    let (u32_t, u64_t) = (PrimitiveType::U32.into(), PrimitiveType::U64.into());
    let mut ctx = TypeCheckerCtx::new();
    let errors = ThreeOperandsChecker::new(&u32_t, &u64_t, &u64_t, &mut ctx)
        .same_operands()
        .all_same()
        .get()
        .expect_err("u32 and u64 differ");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(tags_of(&errors[0]), ["result", "op1, op2"]);
}

#[test]
fn test_all_same_checks_a_single_type() {
    let i8_t = PrimitiveType::I8.into();
    let mut ctx = TypeCheckerCtx::new();
    let t = ThreeOperandsChecker::new(&i8_t, &i8_t, &i8_t, &mut ctx)
        .all_same()
        .signed()
        .get()
        .expect("all i8");
    assert_eq!(t, SignedType::I8);
}

#[test]
fn test_parent_continues_the_operand_check() {
    let u64_t = PrimitiveType::U64.into();
    let mut ctx = TypeCheckerCtx::new();
    let types = TwoOperandsChecker::new(&u64_t, &u64_t, &mut ctx)
        .all_same()
        .unsigned()
        .parent()
        .op(|t| t.integer())
        .get()
        .expect("both u64");
    assert_eq!(
        types,
        TwoPrimitiveTypes {
            result: PrimitiveType::U64,
            op: IntegerType::U64,
        }
    );
}

#[test]
fn test_parent_continues_the_pointed_check() {
    let arr = PointedType::s_arr(PrimitiveType::U8, 4);
    let t = VmType::from(PointedType::ref_reference(arr));
    let errors = root_checker(&t)
        .any_ref()
        .to(|t| {
            t.s_arr()
                .with_len(|len| len > 8, |_, tag| format!("{tag} is too short"))
                .parent()
        })
        .get()
        .expect_err("the array is too short");
    assert!(
        matches!(errors[..], [TypeError::Condition(..)]),
        "{errors:?}"
    );
}

#[test]
fn test_each_of_three_operands_keeps_its_narrowed_type() {
    let (bool_t, i16_t, f64_t) = (
        PrimitiveType::Bool.into(),
        PrimitiveType::I16.into(),
        PrimitiveType::F64.into(),
    );
    let mut ctx = TypeCheckerCtx::new();
    let types = ThreeOperandsChecker::new(&bool_t, &i16_t, &f64_t, &mut ctx)
        .result(|t| t.bool())
        .op1(|t| t.signed())
        .op2(|t| t.float())
        .get()
        .expect("bool, signed and float");
    assert_eq!(
        types,
        ThreePrimitiveTypes {
            result: PrimitiveType::Bool,
            op1: SignedType::I16,
            op2: FloatType::F64,
        }
    );
}
