#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{inherent::Vec, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
		traits::Zero,
	};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}

	/// Local Storage 作用：
	/// - Offchain Worker 可直接读写Local Storage
	/// - 链上代码可通过 Indexing 功能直接向 Local Storage 写数据但是不能读
	///   (可信源的数据可以流想不可信源, 但是可信源不能从不可信源获取数据)
	/// - 可用于Offchain Worker tasks之间的通信和协调, 多个offchain Worker可同时存在, 并通过Local
	///   Storage里
	/// 的数据进行同步, 所以有些有并发访问的数据需要lock

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("OCW ==> Hello World from offchain workers!: {:?}", block_number);

			/*
			// 出块时间是6s, 这里睡眠8s来让offchain worker跨块执行
			let timeout =
				sp_io::offchain::timestamp().add(sp_runtime::offchain::Duration::from_millis(8000));

			sp_io::offchain::sleep_until(timeout);
			 */

			// 在奇数块写, 偶数块读
			if block_number % 2u32.into() != Zero::zero() {
				// odd
				let key = Self::derive_key(block_number);
				let val_ref = StorageValueRef::persistent(&key);

				// get a local random value
				let random_slice = sp_io::offchain::random_seed();

				// get a local timestamp
				let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();

				// combine to a tuple and print it
				let value = (random_slice, timestamp_u64);
				log::info!("OCW ==> in odd block, value to write: {:?}", value);

				struct StateError;
				// write or mutate tuple content to key
				// val_ref.set(&value);
				let res = val_ref.mutate(|val: Result<Option<([u8;32], u64)>, StorageRetrievalError>| -> Result<_, StateError> {
					match val {
						Ok(Some(_)) => Ok(value),
						_ => Ok(value),
					}
				});

				match res {
					Ok(value) => {
						log::info!("OCW ==> in odd block, mutate successfully: {:?}", value);
					},
					Err(MutateStorageError::ValueFunctionFailed(_)) => (),
					Err(MutateStorageError::ConcurrentModification(_)) => (),
				}
			} else {
				// even
				let key = Self::derive_key(block_number - 1u32.into());
				let mut val_ref = StorageValueRef::persistent(&key);

				// get from db by key
				if let Ok(Some(value)) = val_ref.get::<([u8; 32], u64)>() {
					log::info!("OCW ==> in even block, value read: {:?}", value);
					// delete that key
					val_ref.clear();
				}
			}

			log::info!("OCW ==> Leave from offchain workers!: {:?}", block_number);
		}

		/*
		// 块初始化时候执行
		// 在 Starting consensus session on top of parent pre_hash 之后开始执行
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			log::info!("OCW ==> in on_initialize!");
			Weight::from_parts(0, 0)
		}

		// 块确定之后执行
		// 在on_idle之后Prepared block for proposing at block_number之前执行该函数
		fn on_finalize(_n: T::BlockNumber) {
			log::info!("OCW ==> in on_finalize!");
		}

		// 当块runtime中extrinsic执行完毕, 还有剩余时间时执行
		// 在 Starting consensus session on top of parent pre_hash 之后开始执行 on_initialize
		// 之后开始执行块里的交易, 执行完交易之后还有时间的话就会执行on_idle了
		fn on_idle(_n: T::BlockNumber, _remaining_weight: Weight) -> Weight {
			log::info!("OCW ==> in on_idle!");
			Weight::from_parts(0, 0)
		}
		 */
	}

	impl<T: Config> Pallet<T> {
		#[deny(clippy::clone_duble_ref)]
		fn derive_key(block_number: T::BlockNumber) -> Vec<u8> {
			block_number.using_encoded(|encoded_bn| {
				b"node-template::storage::"
					.iter()
					.chain(encoded_bn)
					.copied()
					.collect::<Vec<u8>>()
			})
		}
	}
}
