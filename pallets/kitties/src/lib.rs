#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure,
					traits::{Randomness,Get}};
use frame_system::{ensure_signed};
use sp_runtime::DispatchError;
use sp_io::hashing::blake2_128;
use sp_std::prelude::*;

//定义kitty索引
type KittyIndex = u32;

//定义kitty元组结构体
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

//kitty parent信息
#[derive(Encode,Decode)]
pub struct KittyParent{
	pub father:KittyIndex,
	pub mother:KittyIndex,
}

//kitty相关信息
#[derive(Encode,Decode)]
pub struct KittyFamily{
	pub parent:KittyParent,
	pub wife:KittyIndex,
	pub brothers:Vec<KittyIndex>,
	pub children:Vec<KittyIndex>,
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	//随机数
	type Randomness: Randomness<Self::Hash>;
    type KittyIndex: Get<u32>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
	    //kitty map映射
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) KittyIndex => Option<Kitty>;
		//kitty数量
		pub KittiesCount get(fn kitties_count): KittyIndex;
		//kitty与所有者映射关系
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) KittyIndex => Option<T::AccountId>;
	    //通过账户拥有的kitty个数
	    pub OwnedKittiesCount get(fn owned_kitties_count): map hasher(blake2_128_concat) T::AccountId => u32;


	    //账户下所有的kitty
	    pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat)  T::AccountId => Vec<KittyIndex>;
	    //kitty成员信息（parent,wife,brothers,children）
	    pub KittyFamilyInfo get(fn kitty_faily_info): map hasher(blake2_128_concat) KittyIndex => Option<KittyFamily>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
	    //创建kitty事件
		Created(AccountId, KittyIndex),
		//transfer kitty事件
		Transferred(AccountId, AccountId, KittyIndex),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverFlow,
		InvalidKittyId,
		RrquireDifferentParent,
		NotKittyOwner
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

        // 创建kitty
		#[weight = 100]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?; // 取id
			let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);

            Self::insert_kitty(&sender, kitty_id, kitty);
            Self::update_kitty_owner_count(&sender,true);
			Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}

         //transfer kitty
		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: KittyIndex){
            let sender = ensure_signed(origin)?;
            let account_id = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?; // todo course bug，没有验证所有者
            ensure!(account_id == sender.clone(), Error::<T>::NotKittyOwner);
            <KittyOwners<T>>::insert(kitty_id, to.clone());
            Self::update_kitty_owner_count(&sender,true);
            Self::update_kitty_owner_count(to,false);
			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}

        // 孕育kitty
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex){
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}

	}
}

//计算dna
fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	(selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
	/// 孕育
	fn do_breed(sender: &T::AccountId, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) -> sp_std::result::Result<KittyIndex, DispatchError> {
		// 验证两个Kitty是否存在
		let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
		// 两个kitty是否相同
		ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RrquireDifferentParent);
		// 下个kitty 索引
		let kitty_id = Self::next_kitty_id()?;
		let kitty_1_dna = kitty1.0;
		let kitty_2_dna = kitty2.0;
		// 计算kitty dna
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];
		for i in 0..kitty_1_dna.len() {
			new_dna[i] = combine_dna(kitty_1_dna[i], kitty_2_dna[i], selector[i]);
		}
		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));
		Ok(kitty_id)
	}

	// 插入
	fn insert_kitty(owner: &T::AccountId, kitty_id: KittyIndex, kitty: Kitty) {
		Kitties::insert(kitty_id, kitty);
		KittiesCount::put(kitty_id + 1);
		<KittyOwners<T>>::insert(kitty_id, owner);
	}

	//计算下一个kitty索引
	fn next_kitty_id() -> sp_std::result::Result<KittyIndex, DispatchError> {
		let kitty_id = Self::kitties_count(); // 获取
		if kitty_id == KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverFlow.into());
		}
		Ok(kitty_id)
	}

	//随机值
	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = ( // hash data
						T::Randomness::random_seed(),
						&sender,
						<frame_system::Module<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128) // 128 bit
	}

	//更新拥有者拥有的kitty数量
	fn update_kitty_owner_count(owner:&T::AccountId,is_add:bool){
		owned_kitties_count::get(owner);
		let (kitty_count,_)= OwnedKittiesCount::<T>::get(AccountId);
		if is_add {
			OwnedKittiesCount::put(owner,kitty_count+1);
		}else{
			OwnedKittiesCount::put(owner,kitty_count-1);
		}
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::H256;
	use frame_support::{impl_outer_origin, parameter_types, weights::Weight, assert_ok, assert_noop,
						traits::{OnFinalize, OnInitialize},
	};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
	};
	use frame_system as system;

	impl_outer_origin! {
	    pub enum Origin for Test {}
    }

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

	impl system::Trait for Test {
		type BaseCallFilter = ();
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type DbWeight = ();
		type BlockExecutionWeight = ();
		type ExtrinsicBaseWeight = ();
		type MaximumExtrinsicWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type PalletInfo = ();
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
	}

	type Randomness = pallet_randomness_collective_flip::Module<Test>;

	impl Trait for Test {
		type Event = ();
		type Randomness = Randomness;
	}

	pub type Kitties = Module<Test>;
	pub type System = frame_system::Module<Test>;

	fn run_to_block(n: u64) {
		while System::block_number() < n {
			Kitties::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
			System::set_block_number(System::block_number() + 1);
			System::on_initialize(System::block_number());
			Kitties::on_initialize(System::block_number());
		}
	}

	pub fn new_test_ext() -> sp_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

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



}
