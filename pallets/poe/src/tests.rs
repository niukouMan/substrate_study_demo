use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_claim_works(){
	new_test_ext().execute_with(||{
		let claim :Vec<u8> = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1),claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim),(1,frame_system::Module::<Test>::block_number()))
	})
}


#[test]
fn create_claim_failed_when_claim_already_exist(){
	new_test_ext().execute_with(||{
		let claim :Vec<u8> = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1),claim.clone());
		assert_noop!(PoeModule::create_claim(Origin::signed(1),claim.clone()),
		Error::<Test>::DuplicateClaim);

	})
}


#[test]
fn revoke_claim_works(){
	new_test_ext().execute_with(||{
		let claim:Vec<u8> = vec![0,1];
		let _= PoeModule::create_claim(Origin::signed(1),claim.clone());
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1),claim.clone()));
	})
}


#[test]
fn revoke_claim_failed_when_claim_is_not_exist(){
	new_test_ext().execute_with(||{
		let claim:Vec<u8> = vec![0,1];
		assert_noop!(PoeModule::revoke_claim(Origin::signed(1),claim.clone()),
		Error::<Test>::ClaimNotExist);
	})
}


#[test]
fn send_claim_works(){
	new_test_ext().execute_with(||{
		let account_id:u64 = 1222;
		let claim:Vec<u8> = vec![0,1];
		PoeModule::create_claim(Origin::signed(1),claim.clone());
		assert_ok!(PoeModule::send_claim(Origin::signed(1),claim.clone(),account_id));
	});
}

#[test]
fn send_claim_failed_when_claim_not_exist(){
	new_test_ext().execute_with(||{
		let account_id:u64 = 1222;
		let claim:Vec<u8> = vec![0,1];
		assert_ok!(PoeModule::send_claim(Origin::signed(1),claim.clone(),account_id));
	});
}



