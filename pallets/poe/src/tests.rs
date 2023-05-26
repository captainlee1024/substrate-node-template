use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn create_claim_should_works() {
	test_create_claim_should_works();
}

#[test]
fn create_claim_failed_when_not_signed() {}

#[test]
fn create_claim_failed_when_already_exist() {
	test_create_claim_failed_when_already_exist();
}

#[test]
fn revoke_claim_should_works() {
	test_revoke_claim_should_works();
}

#[test]
fn revoke_claim_failed_when_not_exist() {
	test_revoke_claim_failed_when_not_exist();
}

#[test]
fn revoke_claim_failed_when_not_owner() {
	test_revoke_claim_failed_when_not_owner();
}

#[test]
fn transfer_claim_should_works() {
	test_transfer_claim_should_works();
}

#[test]
fn transfer_claim_failed_when_not_exist() {
	test_transfer_claim_failed_when_not_exist();
}

#[test]
fn transfer_claim_failed_when_not_owner() {
	test_transfer_claim_failed_when_not_owner()
}

fn test_create_claim_should_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));

		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		)
	})
}

fn test_revoke_claim_should_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));

		assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(1), claim.clone()));

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
	})
}

fn test_create_claim_failed_when_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));

		assert_noop!(
			PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

fn test_revoke_claim_failed_when_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}

fn test_revoke_claim_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));

		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

fn test_transfer_claim_should_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);

		assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		)
	})
}

fn test_transfer_claim_failed_when_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2),
			Error::<Test>::ClaimNotExist
		);
	})
}

fn test_transfer_claim_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));

		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(3), claim.clone(), 2),
			Error::<Test>::NotClaimOwner
		);
	})
}
