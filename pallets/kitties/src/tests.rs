use crate::{Event, Error, mock::*};
use frame_support::{assert_noop, assert_ok, traits::{OnFinalize, OnInitialize}};

fn run_to_block( n: u64) {
	while System::block_number() < n {
		KittiesModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
		KittiesModule::on_initialize(System::block_number());
	}
}

// 测试创建一个 Kitty
#[test]
fn create_kitty_works(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create( Origin::signed(1)) );
	})
}

// 测试 质押不足
#[test]
fn create_kitty_failed_when_not_enough_money(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_noop!(KittiesModule::create( Origin::signed(10)) , Error::<Test>::MoneyNotEnough);
	})
}

// 测试转让 Kitty
#[test]
fn transfer_kitty_works(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_ok!(KittiesModule::transfer( Origin::signed(1), 2, 0 ) );

		assert_eq!(
			System::events()[4].event,
			TestEvent::kitties( Event::<Test>::Transferred( 1u64 ,2u64, 0) )
		);
	});
}

// 测试转让，kitty不存在情况
#[test]
fn transfer_kitty_failed_when_not_exists(){
	new_test_ext().execute_with(|| {
		assert_noop!(KittiesModule::transfer( Origin::signed(1), 2, 0 ) , Error::<Test>::KittyNotExists);
	})
}

// 测试转让，因为不是kitty拥有者情况
#[test]
fn transfer_kitty_failed_when_not_owner(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!(KittiesModule::transfer( Origin::signed(2), 3, 0 ) , Error::<Test>::NotKittyOwner);
	})
}

// 测试繁殖
#[test]
fn breed_kitty_work(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_ok!( KittiesModule::breed( Origin::signed(1), 0, 1 ) );
	});
}

// 测试繁殖，两只猫相同情况
#[test]
fn breed_kitty_fail_when_same(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!( KittiesModule::breed( Origin::signed(1), 0, 0 ) , Error::<Test>::RequiredDiffrentParent);
	})
}

// 测试繁殖，其中一只猫不存在情况
#[test]
fn breed_kitty_fail_when_not_exists(){
	new_test_ext().execute_with(|| {
		let _ = KittiesModule::create( Origin::signed(1) );
		assert_noop!( KittiesModule::breed( Origin::signed(1), 0, 1 ) , Error::<Test>::KittyNotExists);
	})
}

