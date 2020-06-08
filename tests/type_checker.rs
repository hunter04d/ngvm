#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne};

use ngvm::types::checker::{HasTypeCheckerCtx, TypeChecker, TypeCheckerCtx};
use ngvm::types::{PointedType, PrimitiveType, RefLocation, VmType};

fn root_checker(t: &VmType) -> TypeChecker<'_, TypeCheckerCtx> {
    TypeChecker {
        tag: "test".into(),
        vm_type: Some(t),
        ctx: Default::default(),
    }
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
