use crate::{
	mock::*,
	Error,
	Event::{KittyBreed, KittyCreated, KittyTransferred},
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

mod create_kitty {
	use super::*;

	mod success {
		use super::*;

		#[test]
		fn creat_should_work() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				let account_id = 1;
				assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id)));

				assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
				assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
				assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
				assert_eq!(KittiesModule::kitty_parents(kitty_id), None);

				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);
				assert_noop!(
					KittiesModule::creat(RuntimeOrigin::signed(account_id)),
					Error::<Test>::KittyIdOverflow
				);

				let kitty = KittiesModule::kitties(kitty_id).expect("kitty was created");
				// homework
				System::assert_last_event(KittyCreated { who: account_id, kitty_id, kitty }.into());
			})
		}
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				assert_noop!(KittiesModule::creat(RuntimeOrigin::root()), BadOrigin);
			})
		}

		#[test]
		fn next_kitty_id_overflow() {
			new_test_ext().execute_with(|| {
				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);
				assert_noop!(
					KittiesModule::creat(RuntimeOrigin::signed(0)),
					Error::<Test>::KittyIdOverflow,
				);
			})
		}
	}
}

mod breed_kitty {
	use super::*;

	mod success {
		use super::*;

		#[test]
		fn breed_should_work() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				let account_id = 0;

				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
					Error::<Test>::SameKittyId
				);

				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1),
					Error::<Test>::InvalidKittyId
				);

				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id)));
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id)));

				assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

				assert_ok!(KittiesModule::breed(
					RuntimeOrigin::signed(account_id),
					kitty_id,
					kitty_id + 1
				));

				let breed_kitty_id = 2;
				assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1);
				assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true);
				assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));
				assert_eq!(
					KittiesModule::kitty_parents(breed_kitty_id),
					Some((kitty_id, kitty_id + 1))
				);

				let kitty = KittiesModule::kitties(kitty_id + 2).expect("child kitty was created");
				// homework
				System::assert_last_event(
					KittyBreed { who: account_id, kitty_id: kitty_id + 2, kitty }.into(),
				);
			})
		}
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				assert_noop!(KittiesModule::breed(RuntimeOrigin::root(), 0, 1), BadOrigin);
			})
		}

		#[test]
		fn parents_are_same_kitty() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(0), 0, 0),
					Error::<Test>::SameKittyId
				);
			})
		}

		#[test]
		fn parent_not_found() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(0), 0, 1),
					Error::<Test>::InvalidKittyId
				);
			})
		}

		#[test]
		fn next_kitty_id_overflow() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id)));
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id)));
				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);

				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(0), 0, 1),
					Error::<Test>::KittyIdOverflow,
				);
			})
		}
	}
}

mod transfer_kitty {
	use super::*;

	mod success {
		use super::*;

		#[test]
		fn transfer_should_work() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				let account_id = 1;
				let recipient = 2;
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id)));
				assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

				assert_noop!(
					KittiesModule::transfer(RuntimeOrigin::signed(recipient), recipient, kitty_id,),
					Error::<Test>::NotOwner
				);

				assert_ok!(KittiesModule::transfer(
					RuntimeOrigin::signed(account_id),
					recipient,
					kitty_id
				));

				assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient));

				assert_ok!(KittiesModule::transfer(
					RuntimeOrigin::signed(recipient),
					account_id,
					kitty_id
				));

				assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

				// homework
				System::assert_last_event(
					KittyTransferred { who: recipient, recipient: account_id, kitty_id }.into(),
				);
			})
		}
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				assert_noop!(KittiesModule::transfer(RuntimeOrigin::root(), 0, 0), BadOrigin);
			})
		}

		#[test]
		fn kitty_not_found() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					KittiesModule::transfer(RuntimeOrigin::signed(0), 1, 0),
					Error::<Test>::InvalidKittyId
				);
			})
		}

		#[test]
		fn not_owner() {
			new_test_ext().execute_with(|| {
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(2)));
				assert_noop!(
					KittiesModule::transfer(RuntimeOrigin::signed(0), 1, 0),
					Error::<Test>::NotOwner
				);
			})
		}
	}
}
