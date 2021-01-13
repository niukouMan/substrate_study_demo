#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module,decl_storage, decl_event, decl_error, StorageValue, ensure, StorageMap, Parameter,
					traits::{Currency,Randomness,Get, ReservableCurrency}
};
use frame_system::{ensure_signed};
use sp_io::hashing::blake2_128;
use sp_std::prelude::*;
use sp_runtime::{DispatchError,traits::{AtLeast32Bit,Bounded}};

//定义kitty索引
//type KittyIndex = u32;

//定义kitty元组结构体
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

//kitty parent信息
//#[derive(Encode,Decode)]
//pub struct KittyParent<T: Trait>{
//	pub father:T::KittyIndex,
//	pub mother:T::KittyIndex,
//}

//kitty相关信息
//#[derive(Encode,Decode)]
//pub struct KittyFamily<T: Trait>{
//	pub parent:KittyParent,
//	pub wife:T::KittyIndex,
//	pub brothers:Vec<T::KittyIndex>,
//	pub children:Vec<T::KittyIndex>,
//}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	//随机数
	type Randomness: Randomness<Self::Hash>;
	type KittyIndex: Parameter + AtLeast32Bit + Bounded + Default + Copy;
	type NewKittyReserve: Get<BalanceOf<Self>>;
	// 用于质押等于资产
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
	    //kitty map映射
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
		//kitty数量
		pub KittiesCount get(fn kitties_count): T::KittyIndex;
		//kitty与所有者映射关系
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
		// 记录某个拥有者与猫之间的关系
		pub OwnedKitties get(fn owned_kitties):double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::KittyIndex => Option<T::KittyIndex>;

	    //kitty成员信息（parent,wife,brothers,children）
	   // pub KittyFamilyInfo get(fn kitty_faily_info): map hasher(blake2_128_concat) T::KittyIndex => Option<KittyFamily<T>>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, KittyIndex = <T as Trait>::KittyIndex {
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
		NotKittyOwner,
		MoneyNotEnough,
		UnReserveMoneyNotEnough,
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
            //质押资产
			T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;
            Self::insert_kitty(&sender, kitty_id, kitty);
			Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}

         //transfer kitty
		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex){
            let sender = ensure_signed(origin)?;
            let account_id = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            ensure!(account_id == sender.clone(), Error::<T>::NotKittyOwner);

            // 质押被转让人的代币
			T::Currency::reserve(&to, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;
			// 解质押转出人的代币
			T::Currency::unreserve(&sender, T::NewKittyReserve::get());

            <KittyOwners<T>>::insert(kitty_id, to.clone());
            // 从之前的拥有人中删除关系
			OwnedKitties::<T>::remove(&sender, kitty_id);
			OwnedKitties::<T>::insert(&to, kitty_id, kitty_id);
			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}

        // 孕育kitty
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex){
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
	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
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
	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		<Kitties::<T>>::insert(kitty_id, kitty);
		<KittiesCount::<T>>::put(kitty_id + 1.into());
		<KittyOwners::<T>>::insert(kitty_id, owner);
        // 保存拥有者拥有的 Kitty 数据
        <OwnedKitties::<T>>::insert(owner, kitty_id, kitty_id);
        //todo 保存kitty家庭成员关系


	}

	//计算下一个kitty索引
	fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError>{
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
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

  }
