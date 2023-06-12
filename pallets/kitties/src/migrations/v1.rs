use frame_support::{
	pallet_prelude::*, storage::StoragePrefixedMap, traits::GetStorageVersion, weights::Weight,
};

use crate::{Config, Kitties, Kitty, KittyId, Pallet};
use frame_support::{migration::storage_key_iter, Blake2_128Concat};
use frame_system::pallet_prelude::*;

// 升级需要把老的结构体先拿过来, 读取数据然后构造新数据赋值给新数据
//
// 需要一个保存Kitty主题的结构
// 要在链上存储需要一些特这个比如Encode Decode TypeInfo MaxEncodeLen...
#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct OldKitty(pub [u8; 16]);

pub fn migrate<T: Config>() -> Weight {
	// 获取链上的storage version
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	// 获取当前的storage version
	let current_version = Pallet::<T>::current_storage_version();

	// 当前的v1版本只适用与从v0到v1的升级
	if on_chain_version != 0 {
		return Weight::zero()
	}

	if current_version != 1 {
		return Weight::zero()
	}

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();
	// 数据迁移, 删除旧数据添加新数据, 对于一些之前没有的信息字段赋予默认值
	for (kitty_id, kitty) in
		storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain()
	{
		let new_kitty = Kitty { dna: kitty.0, name: *b"abcd" };

		Kitties::<T>::insert(kitty_id, &new_kitty);
	}

	return Weight::zero()
}
