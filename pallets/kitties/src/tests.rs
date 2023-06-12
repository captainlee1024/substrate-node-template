use crate::{
	mock::*,
	Error,
	Event::{KittyBought, KittyBreed, KittyCreated, KittyOnSale, KittyTransferred},
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
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));

				assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));

				assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
				assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
				assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
				assert_eq!(KittiesModule::kitty_parents(kitty_id), None);

				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);
				assert_noop!(
					KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name),
					Error::<Test>::KittyIdOverflow
				);

				let kitty = KittiesModule::kitties(kitty_id).expect("kitty was created");
				System::assert_last_event(KittyCreated { who: account_id, kitty_id, kitty }.into());
			})
		}
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_noop!(KittiesModule::creat(RuntimeOrigin::root(), kitty_name), BadOrigin);
			})
		}

		#[test]
		fn next_kitty_id_overflow() {
			new_test_ext().execute_with(|| {
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);
				assert_noop!(
					KittiesModule::creat(RuntimeOrigin::signed(0), kitty_name),
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
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));
				assert_noop!(
					KittiesModule::breed(
						RuntimeOrigin::signed(account_id),
						kitty_id,
						kitty_id,
						kitty_name
					),
					Error::<Test>::SameKittyId
				);

				assert_noop!(
					KittiesModule::breed(
						RuntimeOrigin::signed(account_id),
						kitty_id,
						kitty_id + 1,
						kitty_name
					),
					Error::<Test>::InvalidKittyId
				);

				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));

				assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

				assert_ok!(KittiesModule::breed(
					RuntimeOrigin::signed(account_id),
					kitty_id,
					kitty_id + 1,
					kitty_name
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
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::root(), 0, 1, kitty_name),
					BadOrigin
				);
			})
		}

		#[test]
		fn parents_are_same_kitty() {
			new_test_ext().execute_with(|| {
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(0), 0, 0, kitty_name),
					Error::<Test>::SameKittyId
				);
			})
		}

		#[test]
		fn parent_not_found() {
			new_test_ext().execute_with(|| {
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(0), 0, 1, kitty_name),
					Error::<Test>::InvalidKittyId
				);
			})
		}

		#[test]
		fn next_kitty_id_overflow() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));

				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));
				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);

				assert_noop!(
					KittiesModule::breed(RuntimeOrigin::signed(0), 0, 1, kitty_name),
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
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));
				assert_ok!(Balances::set_balance(RuntimeOrigin::root().into(), recipient, 100, 0));

				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));
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
				let account_id = 2;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(2), kitty_name));
				assert_noop!(
					KittiesModule::transfer(RuntimeOrigin::signed(0), 1, 0),
					Error::<Test>::NotOwner
				);
			})
		}
	}
}

mod sale_kitty {
	use super::*;

	mod success {
		use super::*;

		#[test]
		fn sale_should_work() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				let kitty_id = 0;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				// 设置一些账户余额用于转账
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));

				assert_ok!(KittiesModule::creat(
					RuntimeOrigin::signed(account_id).into(),
					kitty_name
				));
				assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
				System::assert_last_event(KittyOnSale { who: account_id, kitty_id }.into());
			})
		}
	}

	mod failed_when {
		use super::*;

		#[test]
		fn kitty_not_found() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					KittiesModule::sale(RuntimeOrigin::signed(0).into(), 0),
					Error::<Test>::InvalidKittyId
				);
			});
		}

		#[test]
		fn not_owner() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				let kitty_id = 0;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				// 设置一些账户余额用于转账
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));

				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));

				assert_noop!(
					KittiesModule::sale(RuntimeOrigin::signed(1), kitty_id),
					Error::<Test>::NotOwner,
				);
			});
		}

		#[test]
		fn already_on_sale() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				let kitty_id = 0;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				// 设置一些账户余额用于转账
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));
				assert_ok!(KittiesModule::creat(RuntimeOrigin::signed(account_id), kitty_name));

				assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
				assert_noop!(
					KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
					Error::<Test>::AlreadyOnSale,
				);
			});
		}
	}
}

mod buy_kitty {
	use super::*;

	mod success {
		use super::*;

		#[test]
		fn buy_should_work() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				let account_id2 = 1;
				let kitty_id = 0;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				// 设置一些账户余额用于转账
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id2,
					1000000,
					1000000
				));

				assert_ok!(KittiesModule::creat(
					RuntimeOrigin::signed(account_id).into(),
					kitty_name
				));
				assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
				System::assert_last_event(KittyOnSale { who: account_id, kitty_id }.into());

				assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(account_id2).into(), kitty_id));
				System::assert_last_event(KittyBought { who: account_id2, kitty_id }.into());
			})
		}
	}

	mod failed_when {
		use super::*;

		#[test]
		fn kitty_not_found() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					KittiesModule::buy(RuntimeOrigin::signed(0), 0),
					Error::<Test>::InvalidKittyId,
				);
			});
		}

		#[test]
		fn already_owned() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				let kitty_id = 0;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				// 设置一些账户余额用于转账
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));

				assert_ok!(KittiesModule::creat(
					RuntimeOrigin::signed(account_id).into(),
					kitty_name
				));
				assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
				System::assert_last_event(KittyOnSale { who: account_id, kitty_id }.into());

				assert_noop!(
					KittiesModule::buy(RuntimeOrigin::signed(account_id).into(), kitty_id),
					Error::<Test>::AlreadyOwned,
				);
			})
		}

		#[test]
		fn not_on_sale() {
			new_test_ext().execute_with(|| {
				let account_id = 0;
				let account_id2 = 1;
				let kitty_id = 0;
				// let kitty_name = *b"abcd"
				let kitty_name = *b"abcdefgh";

				// 设置一些账户余额用于转账
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id,
					1000000,
					1000000
				));
				assert_ok!(Balances::set_balance(
					RuntimeOrigin::root().into(),
					account_id2,
					1000000,
					1000000
				));

				assert_ok!(KittiesModule::creat(
					RuntimeOrigin::signed(account_id).into(),
					kitty_name
				));

				assert_noop!(
					KittiesModule::buy(RuntimeOrigin::signed(account_id2).into(), kitty_id),
					Error::<Test>::NotOnSale,
				);
			});
		}
	}
}
