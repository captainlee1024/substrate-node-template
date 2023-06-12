use frame_support::{
	pallet_prelude::*, storage::StoragePrefixedMap, traits::GetStorageVersion, weights::Weight,
};

use crate::{Config, Kitties, Kitty, KittyId, Pallet};
use frame_support::{migration::storage_key_iter, Blake2_128Concat};

#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct KittyVersion0(pub [u8; 16]);

#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct KittyVersion1 {
	// 原来的数据
	pub dna: [u8; 16],
	// 新增名字
	pub name: [u8; 4],
}

pub fn migrate<T: Config>() -> Weight {
	// 获取链上的storage version
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	// 获取当前的storage version
	let current_version = Pallet::<T>::current_storage_version();

	// 当前的v2版本只能从v0或者v1升级上来
	if on_chain_version != 0 && on_chain_version != 1 {
		return Weight::zero()
	}

	// 升级到v2
	if current_version != 2 {
		return Weight::zero()
	}

	if on_chain_version == 0 {
		let module = Kitties::<T>::module_prefix();
		let item = Kitties::<T>::storage_prefix();
		// 数据迁移, 删除旧数据添加新数据, 对于一些之前没有的信息字段赋予默认值
		for (kitty_id, kitty) in
			storage_key_iter::<KittyId, KittyVersion0, Blake2_128Concat>(module, item).drain()
		{
			let new_kitty = Kitty { dna: kitty.0, name: *b"abcdefgh" };

			Kitties::<T>::insert(kitty_id, &new_kitty);
		}
	}
	if on_chain_version == 1 {
		let module = Kitties::<T>::module_prefix();
		let item = Kitties::<T>::storage_prefix();
		// 数据迁移, 删除旧数据添加新数据, 对于一些之前没有的信息字段赋予默认值
		for (kitty_id, kitty) in
			storage_key_iter::<KittyId, KittyVersion1, Blake2_128Concat>(module, item).drain()
		{
			let mut new_name = [0u8; 8];
			new_name[0..4].copy_from_slice(&kitty.name[..4]);
			let new_kitty = Kitty { dna: kitty.dna, name: new_name };
			Kitties::<T>::insert(kitty_id, &new_kitty);
		}
	}

	return Weight::zero()
}
