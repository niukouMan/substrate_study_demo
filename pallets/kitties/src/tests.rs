use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

// 创建kitty
#[test]
fn owned_kitties_can_append_values() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(Kitties::create(Origin::signed(1)), Ok(()))
	})
}

// transfer kitty
#[test]
fn transfer_kitties() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(Kitties::create(Origin::signed(1)));
		let id = Kitties::kitties_count();
		assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id-1));
		assert_noop!(
                Kitties::transfer(Origin::signed(1), 2, id-1),
                Error::<Test>::NotKittyOwner
                );
	})
}
