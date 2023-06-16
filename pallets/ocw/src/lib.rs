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

use codec::{Decode, Encode};
use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendSignedTransaction, SignedPayload, Signer, SigningTypes,
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);
	pub struct OcwAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
		type RuntimeAppPublic = Public;
		type GenericPublic = sp_core::sr25519::Public;
		type GenericSignature = sp_core::sr25519::Signature;
	}

	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for OcwAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericPublic = sp_core::sr25519::Public;
		type GenericSignature = sp_core::sr25519::Signature;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{inherent::Vec, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use serde::{Deserialize, Deserializer};
	use sp_runtime::{
		offchain::{
			http,
			storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
			Duration,
		},
		traits::Zero,
	};
	use sp_std::vec;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct Payload<Public> {
		number: u64,
		/// signer的公钥
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	// 处理获取到的Github上的数据
	#[derive(Deserialize, Encode, Decode)]
	struct GithubInfo {
		#[serde(deserialize_with = "de_string_to_bytes")]
		login: Vec<u8>,
		#[serde(deserialize_with = "de_string_to_bytes")]
		blog: Vec<u8>,
		public_repos: u32,
	}

	pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s: &str = Deserialize::deserialize(de)?;
		Ok(s.as_bytes().to_vec())
	}

	// 实现 fmt::Debug 将字节数组转成string更好看一些
	use core::{convert::TryInto, fmt};
	use frame_system::offchain::SendUnsignedTransaction;

	impl fmt::Debug for GithubInfo {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(
				f,
				"{{ login: {}, blog: {}, public_repos: {} }}",
				sp_std::str::from_utf8(&self.login).map_err(|_| fmt::Error)?,
				sp_std::str::from_utf8(&self.blog).map_err(|_| fmt::Error)?,
				&self.public_repos
			)
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
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

		// 提供给ocw调用的extrinsic call
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;
			log::info!("OCW ==> in submit_data call: {:?}", payload);
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn unsigned_extrinsic_with_signed_payload(
			origin: OriginFor<T>,
			payload: Payload<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			log::info!(
				"OCW ==> in call unsigned_extrinsic_with_signed_payload: {:?}",
				payload.number
			);
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transaction are disallowed, but implementing the valdator
		/// here we make sure that some particular calls (the onese produced buy offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const UNSIGNED_TXS_PRIORITY: u64 = 100;
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("my-pallet") // 前缀
					.priority(UNSIGNED_TXS_PRIORITY) // 在交易池中的拍序权重
					.and_provides([&provide])
					.longevity(3) // 交易在交易池中的存活时间
					.propagate(true) // 是否通过网络广播
					.build()
			};

			match call {
				Call::unsigned_extrinsic_with_signed_payload { ref payload, ref signature } => {
					// 验签
					if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
						return InvalidTransaction::BadProof.into()
					}

					valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
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

			// let payload: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
			// _ = Self::send_signed_tx(payload);

			// 我们的数据
			let number: u64 = 42;
			// 获取一个当前pallet的账户
			// 在service里我们就设置了一个账户给当前pallet   账户是Alice
			let signer = Signer::<T, T::AuthorityId>::any_account();

			if let Some((_, res)) = signer.send_unsigned_transaction(
				// this line is to prepare and return payload
				|acct| Payload { number, public: acct.public.clone() },
				|payload, signature| Call::unsigned_extrinsic_with_signed_payload {
					payload,
					signature,
				},
			) {
				match res {
					Ok(()) => {
						log::info!("OCW ==> unsigned tx with signed payload successfully sent.");
					},
					Err(()) => {
						log::error!("OCW ==> sending unsigned tx with signed payload failed.");
					},
				};
			} else {
				log::error!("OCW ==> No local account available");
			}

			log::info!("OCW ==> Leave from offchain workers!: {:?}", block_number);
		}
	}

	impl<T: Config> Pallet<T> {
		fn send_signed_tx(payload: Vec<u8>) -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if !signer.can_sign() {
				return Err(
					"No local acounts available. Consider adding one via `author_insertKey` RPC.",
				)
			}

			let results = signer
				.send_signed_transaction(|_account| Call::submit_data { payload: payload.clone() });

			for (acc, res) in &results {
				match res {
					Ok(()) => log::info!("[{:?}] Submitted data: {:?}", acc.id, payload),
					Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
				}
			}

			Ok(())
		}
	}

	/*
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("OCW ==> Hello World from offchain workers!: {:?}", block_number);

			// if let Ok(info) = Self::fetch_github_info() {
			// 	log::info!("OCW ==> Github Info: {:?}", info);
			// } else {
			// 	log::info!("OCW ==> Error while fetch github info!");
			// }

			match Self::fetch_github_info() {
				Ok(info) => {
					log::info!("OCW ==> Github Info: {:?}", info);
				},
				Err(err) => {
					log::info!("OCW ==> Error while fetch github info, Err: {:?}", err);
				},
			}

			/*
			// 出块时间是6s, 这里睡眠8s来让offchain worker跨块执行
			let timeout =
				sp_io::offchain::timestamp().add(sp_runtime::offchain::Duration::from_millis(8000));

			sp_io::offchain::sleep_until(timeout);
			 */

			/*
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

			 */

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
		fn fetch_github_info() -> Result<GithubInfo, http::Error> {
			// prepare for send request
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
			let request = http::Request::get("https://api.github.com/orgs/substrate-developer-hub");
			let pending = request
				.add_header("User-Agent", "Substrate-Offchain-Worker")
				.deadline(deadline)
				.send()
				.map_err(|_| http::Error::IoError)?;

			// send and waiting resp
			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::info!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}

			// 获取到数据, 将body数据收集到Vec<u8>
			let body = response.body().collect::<Vec<u8>>();
			// 转成str
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::info!("No UTF8 body");
				http::Error::Unknown
			})?;

			// parse the response str
			// serde_json 将str转换成GithubInfo结构体
			let gh_info: GithubInfo =
				serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

			Ok(gh_info)
		}
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

	 */
}
