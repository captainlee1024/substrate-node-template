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

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");
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
		public_repos: u32,
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

	#[derive(Deserialize, Encode, Decode)]
	struct Price {
		pub bitcoin: Bitcoin,
	}

	#[derive(Deserialize, Encode, Decode)]
	struct Bitcoin {
		usd: u32,
	}

	// 实现 fmt::Debug 将字节数组转成string更好看一些
	use core::{convert::TryInto, fmt};
	use frame_system::offchain::SendUnsignedTransaction;

	impl fmt::Debug for Price {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "{{ price(USD): [bitcoin: {}] }}", &self.bitcoin.usd)
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

		/// 通过 offchain indexing 写入到offchain storage
		///
		/// 每5个块更新一次数据, 将获得的数据包装一个不签名带签名负载的交易发送到链上
		/// 然后在链上通过offchain indexing 将数据写入到 offchain storage
		/// 在其他块中的offchain_worker读取一下当前存储的repos
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn set_substrate_developer_hub_pub_repos(
			origin: OriginFor<T>,
			payload: Payload<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			log::info!(
				"OCW HOME WORK ==> in call unsigned_extrinsic_with_signed_payload: {:?}",
				payload.public_repos
			);

			Self::set_local_storage_with_offchain_index(payload.public_repos);

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
				ValidTransaction::with_tag_prefix("ocw-home-work") // 前缀
					.priority(UNSIGNED_TXS_PRIORITY) // 在交易池中的拍序权重
					.and_provides([&provide])
					.longevity(3) // 交易在交易池中的存活时间
					.propagate(true) // 是否通过网络广播
					.build()
			};

			match call {
				Call::set_substrate_developer_hub_pub_repos { ref payload, ref signature } => {
					// 验签
					if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
						return InvalidTransaction::BadProof.into()
					}

					valid_tx(b"set_substrate_developer_hub_pub_repos".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("OCW HOME WORK ==> Hello World from offchain workers!: {:?}", block_number);

			// 每5个块更新一次数据, 将获得的数据包装一个不签名带签名负载的交易发送到链上
			// 然后在链上通过offchain indexing 将数据写入到 offchain storage
			// 在其他块中的offchain_worker读取一下当前存储的repos
			if block_number % 5u32.into() == Zero::zero() || block_number == 1u32.into() {
				// 发起http请求获取外界数据, 这里是github账号的 public repos数量
				let info = match Self::fetch_github_info() {
					Ok(info) => {
						log::info!("OCW HOME WORK ==> Github Info: {:?}", info);
						info
					},
					Err(err) => {
						log::info!(
							"OCW HOME WORK ==> Error while fetch github info, Err: {:?}",
							err
						);
						return
					},
				};

				let public_repos: u32 = info.public_repos;
				// 获取一个当前pallet的账户
				// 在service里我们就设置了一个账户给当前pallet   账户是Alice
				let signer = Signer::<T, T::AuthorityId>::any_account();

				// 发送一个不签名带签名payload的交易到链上, 将http中获取的数据通过Offchain
				// Indexing从链上写入Offchain Storage中
				if let Some((_, res)) = signer.send_unsigned_transaction(
					// this line is to prepare and return payload
					|acct| Payload { public_repos, public: acct.public.clone() },
					|payload, signature| Call::set_substrate_developer_hub_pub_repos {
						payload,
						signature,
					},
				) {
					match res {
						Ok(()) => {
							log::info!(
							"OCW HOME WORK ==> unsigned tx with signed payload successfully sent."
						);
						},
						Err(()) => {
							log::error!(
								"OCW HOME WORK ==> sending unsigned tx with signed payload failed."
							);
						},
					};
				} else {
					log::error!("OCW HOME WORK ==> No local account available");
				}
			} else {
				if let Ok(repos) = Self::get_local_storage() {
					log::info!("OCW HOME WORK ==> Offchain Indexing: substrate developer hub public repos: {:?}", repos);
				} else {
					log::info!("OCW HOME WORK ==> Offchain Indexing: substrate developer hub public repos: unknown");
				}
			}

			// 获取bitcoin价格的api fix之后再使用
			// match Self::fetch_bitcoin_price() {
			// 	Ok(info) => {
			// 		log::info!("OCW HOME WORK ==> Fetch Price: {:?}", info);
			// 	},
			// 	Err(err) => {
			// 		log::info!("OCW HOME WORK ==> Error while fetch Price, Err: {:?}", err);
			// 	},
			// }

			log::info!("OCW HOME WORK ==> Leave from offchain workers!: {:?}", block_number);
		}
	}

	impl<T: Config> Pallet<T> {
		// extrinsic中通过该方法将public repos设置到Offchain Storage中
		fn set_local_storage_with_offchain_index(repos: u32) {
			let key = Self::derived_key();
			sp_io::offchain_index::set(&key, repos.encode().as_slice());
			log::info!("OCW HOME WORK ==> set repos:[{:?}] to offchain storage", repos);
		}

		// 从Offchain Storage中读取链上通过Offchain Indexing设置的值
		fn get_local_storage() -> Result<u32, &'static str> {
			let key = Self::derived_key();
			let some_number_storage = StorageValueRef::persistent(&key);

			if let Ok(Some(number)) = some_number_storage.get::<u32>() {
				Ok(number)
			} else {
				Err("No number in storage.")
			}
		}

		// 给offchain worker使用, 用于获取substrate developer hub 的pub repos数量
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
				log::info!("OCW HOME WORK ==> Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}

			log::info!("OCW HOME WORK ==> Response: {:?}", response);

			// 获取到数据, 将body数据收集到Vec<u8>
			let body = response.body().collect::<Vec<u8>>();
			// 转成str
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::info!("OCW HOME WORK ==> No UTF8 body");
				http::Error::Unknown
			})?;

			// parse the response str
			// serde_json 将str转换成GithubInfo结构体
			let gh_info: GithubInfo =
				serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

			Ok(gh_info)
		}

		/// 一直超时未找到原因, 暂不使用
		fn fetch_bitcoin_price() -> Result<Price, http::Error> {
			// 构造request 来获取bitcoin 的价格
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(12_000));
			let request = http::Request::get(
				"https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd",
			);
			let pending = request
				.add_header("User-Agent", "Substrate-Offchain-Worker")
				.deadline(deadline)
				.send()
				.map_err(|_| http::Error::IoError)?;

			// 等待response
			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::info!("OCW HOME WORK ==> Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}
			log::info!("OCW HOME WORK ==> Response: {:?}", response);

			// 获取到数据, 将body数据收集到Vec[u8]
			let body = response.body().collect::<Vec<u8>>();
			// 转成str
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::info!("OCW HOME WORK ==> No UTF* body");
				http::Error::Unknown
			})?;
			// 解析到Price结构体

			let price: Price = serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

			Ok(price)
		}

		fn derived_key() -> Vec<u8> {
			// sdh-pr substrate-developer-hub-public-repos
			b"offchain-index-sdh-pr::value".encode()
		}
	}
}
