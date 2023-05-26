#![cfg_attr(not(feature = "std"), no_std)]

/// proof of existence
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type MaxClaimLength: Get<u32>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
		ClaimRevoked(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
		ClaimTransferred(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ProofNotExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		) -> DispatchResultWithPostInfo {
			// 交易是否签名
			let sender = ensure_signed(origin)?;

			// 校验是否包含claim, 是否存正已经存在了
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

			Proofs::<T>::insert(
				&claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number()),
			);

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		) -> DispatchResultWithPostInfo {
			// 校验签名
			let sender = ensure_signed(origin)?;

			// 查找claim是否存在
			// 存在获取owner
			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 判断是是否是所有者
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 删除
			Proofs::<T>::remove(&claim);

			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		//转移存证
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
			account_id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			// 校验签名
			let sender = ensure_signed(origin)?;

			// 查询资产
			let (owner, block_number) =
				Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 判断所有者
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 转移资产
			Proofs::<T>::mutate(&claim, |v| *v = Some((account_id, block_number)));

			Self::deposit_event(Event::ClaimTransferred(sender, claim));

			Ok(().into())
		}
	}
}
