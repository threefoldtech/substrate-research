#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use core::{convert::TryInto, fmt};
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::{DispatchError, DispatchResult}, ensure, debug,
    traits::{
		Currency, Get, ExistenceRequirement::AllowDeath, Randomness
    },
    sp_runtime::{
        traits::AccountIdConversion, ModuleId,
        offchain as rt_offchain,
        offchain::{
            storage::StorageValueRef,
            storage_lock::{StorageLock, BlockAndTime},
        },
    }
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
	},
};
use sp_core::{RuntimeDebug, H256};
use sp_std::{
	prelude::*, str
};
use alt_serde::{Deserialize, Deserializer};
use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	// implemented for ocw-runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[cfg(test)]
mod mock;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub enum WorkloadState {
    Created,
    Deployed,
    Cancelled,
}

impl Default for WorkloadState {
    fn default() -> WorkloadState {
        WorkloadState::Created
    }
}

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

const PALLET_ID: ModuleId = ModuleId(*b"Charity!");

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct VolumeType {
    disk_type: u8,
    size: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct Contract<T: Trait> {
    cu_price: u64,
    su_price: u64,
    account_id: T::AccountId,
    node_id: Vec<u8>,
}

impl<T> Default for Contract<T>
    where T: Trait
{
    fn default() -> Contract<T> {
        let account_id = PALLET_ID.into_account();

        Contract {
            cu_price: 0,
            su_price: 0,
            account_id,
            node_id: [0].to_vec(),
        }
    }
}

pub const EXPLORER_NODES: &str = "https://explorer.grid.tf/explorer/nodes/";
pub const EXPLORER_FARMS: &str = "https://explorer.grid.tf/explorer/farms/";
pub const FETCH_TIMEOUT_PERIOD: u64 = 10000; // in milli-seconds
pub const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000; // in milli-seconds
pub const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct NodeInfo {
	// Specify our own deserializing function to convert JSON string to vector of bytes
	#[serde(deserialize_with = "de_string_to_bytes")]
	node_id: Vec<u8>,
	farm_id: u64,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct FarmInfo {
    id: u64,
    threebot_id: u64,
	resource_prices: Vec<ResourcePrice>,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct ResourcePrice {
	currency: u64,
    sru: u64,
    hru: u64,
    cru: u64,
    nru: u64,
	mru: u64,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

impl fmt::Debug for NodeInfo {
	// `fmt` converts the vector of bytes inside the struct back to string for
	//   more friendly display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{ node_id: {}, farm_id: {} }}",
			str::from_utf8(&self.node_id).map_err(|_| fmt::Error)?,
			self.farm_id,
		)
	}
}

pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type RandomnessSource: Randomness<H256>;
    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub ReservationState get(fn reservation_state): map hasher(blake2_128_concat) u64 => WorkloadState;
        pub ReservationsForAccount get(fn reservations_for_account): map hasher(blake2_128_concat) T::AccountId => Vec<u64>;
        pub WorkloadCreated get(fn workloads_created): map hasher (blake2_128_concat) Vec<u8> => Vec<u64>;
        pub WorkloadDeployed get(fn workloads_deployed): map hasher (blake2_128_concat) Vec<u8> => Vec<u64>;
        pub VolumeReservations get(fn volume_reservations): map hasher (blake2_128_concat) u64 => VolumeType;
        pub Contracts get(fn contracts): map hasher (blake2_128_concat) u64 => Contract<T>;
        ReservationID: u64;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {        
        // Will signal a contract has been added for a specific users, for a specific nodeID with a reservationID
        ContractAdded(AccountId, Vec<u8>, u64),
        ContractPaid(AccountId, u64),
        ContractUpdated(AccountId, u64),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        ReservationExists,
        ContractExists,
        ContractNotExists,
        UnknownOffchainMux,
        HttpFetchingError,
        // Error returned when making signed transactions in off-chain worker
		NoLocalAcctForSigning,
        OffchainSignedTxError,
        NoLocalAcctForSignedTx,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn create_contract(origin, node_id: Vec<u8>, volume: VolumeType) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let reservation_id = ReservationID::get();

            ensure!(!Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractExists);

            // Create a contract
            let mut contract = Contract::default();

            contract.node_id = node_id.clone();
            debug::info!("Contract with id: {:?} and nodeId: {:?}", reservation_id, contract.node_id);

            // Create a new accountID based on the reservationID and assign it to the contract
            let account_id = PALLET_ID.into_sub_account(reservation_id);
            let _ = T::Currency::make_free_balance_be(
				&account_id,
				T::Currency::minimum_balance(),
            );
            debug::info!("Assigned accountID: {:?} to contract with id: {:?}", account_id, reservation_id);
            contract.account_id = account_id;

            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);

            // TODO, make generic for each workload type
            VolumeReservations::insert(reservation_id, &volume);
            ReservationID::put(reservation_id + 1);

            ReservationsForAccount::<T>::mutate(&who, |list|  list.push(reservation_id));

            Self::deposit_event(RawEvent::ContractAdded(who, node_id, reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn pay(origin, reservation_id: u64, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let contract = Contracts::<T>::get(reservation_id);

            debug::info!("Transfering: {:?} from {:?} to contract accountId: {:?}", &amount, &who, &contract.account_id);
            // Transfer currency to the contracts account
            T::Currency::transfer(&who, &contract.account_id, amount, AllowDeath)
                .map_err(|_| DispatchError::Other("Can't make transfer"))?;
            
            Self::deposit_event(RawEvent::ContractPaid(contract.account_id, reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn set_contract_price(origin, reservation_id: u64, cu_price: u64, su_price: u64) -> DispatchResult {
            // TODO: Only off chain worker can sign this
            let _ = ensure_signed(origin)?;

            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let mut contract = Contracts::<T>::get(reservation_id);

            contract.cu_price = cu_price;
            contract.su_price = su_price;

            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);
            
            Self::deposit_event(RawEvent::ContractUpdated(contract.account_id, reservation_id));

            Ok(())
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            debug::info!("Entering off-chain worker");
    
            let _ = Self::offchain_signed_tx(block_number);
        }
    }

}

impl<T: Trait> Module<T> {
    fn offchain_signed_tx(block_number: T::BlockNumber) -> Result<(), Error<T>> {
        let mut reservation_id = ReservationID::get();
        if reservation_id > 0 {
            reservation_id -= 1;
        }

        let reservation_id_storage = StorageValueRef::persistent(b"worker::current_reservation_id");
        if let Some(Some(id)) = reservation_id_storage.get::<u64>() {
            // reservationID has already been fetched. Return early.
            if reservation_id == id {
                debug::info!("cached reservation ID: {:?}", id);
                return Ok(());
            }
        }

        let number: u64 = block_number.try_into().unwrap_or(0) as u64;
        let block_hash = <system::Module<T>>::block_hash(block_number);
        debug::info!("Current block is: {:?} (parent: {:?})", number, block_hash);

        debug::info!("Current reservation ID: {:?}", reservation_id);

        let contract = Contracts::<T>::get(reservation_id);
        debug::info!("Contract with ID: {:?} and nodeID: {:?}", reservation_id, contract.node_id);

        
        let farm_info = Self::fetch_farmer_prices(contract.node_id).map_err(|err| {
            debug::info!("{:?}", err); 
            err
        })?;
        
        reservation_id_storage.set(&reservation_id);
        
        // This is the on-chain function
        let cru_price = farm_info.resource_prices[0].cru;
        let sru_price = farm_info.resource_prices[0].sru;
        debug::info!("Prices for farm: {:?}, cru price: {:?}, sru price: {:?}", farm_info.id, cru_price, sru_price);
        
        // retrieve contract account
        let signer = Signer::<T, T::AuthorityId>::any_account();

        let result = signer.send_signed_transaction(|_acct|
            Call::set_contract_price(reservation_id, cru_price, sru_price)
        );
    
        // Display error if the signed tx fails.
        if let Some((acc, res)) = result {
            if res.is_err() {
                debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
                return Err(<Error<T>>::OffchainSignedTxError);
            }
            // Transaction is sent successfully
            return Ok(());
        }
        // The case of `None`: no account is available for sending
        debug::error!("No local account available");
        return Err(<Error<T>>::NoLocalAcctForSignedTx)
    }

	fn fetch_farmer_prices(node_id: Vec<u8>) -> Result<FarmInfo, Error<T>> {
		let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
			b"offchain-explorer::lock", LOCK_BLOCK_EXPIRATION,
			rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION)
		);
        
        if let Ok(_guard) = lock.try_lock() {
			match Self::fetch_n_parse_node(node_id) {
				Ok(node_info) => {
                    match Self::fetch_n_parse_farm(node_info.farm_id) {
                        Ok(farm_info) => {
                            return Ok(farm_info)
                        }
                        Err(err) => { return Err(err); }
                    }
                }
				Err(err) => { return Err(err); }
            };
        }
        return Err(<Error<T>>::HttpFetchingError)
    }
    
	fn fetch_n_parse_node(node_id: Vec<u8>) -> Result<NodeInfo, Error<T>> {
        debug::info!("fetching node");
		let resp_bytes = Self::fetch_node_from_remote(node_id).map_err(|e| {
			debug::error!("fetch_node_from_remote error: {:?}", e);
			<Error<T>>::HttpFetchingError
        })?;

		let resp_str = str::from_utf8(&resp_bytes).map_err(|err| {
            debug::info!("{:?}", err); 
            <Error<T>>::HttpFetchingError
        })?;
        debug::info!("{}", resp_str);

		let node_farm_info: NodeInfo = serde_json::from_str(&resp_str).map_err(|err| {
            debug::info!("{:?}", err); 
            <Error<T>>::HttpFetchingError
        })?;

        debug::info!("got node response");
        Ok(node_farm_info)

    }
    fn fetch_n_parse_farm(farm_id: u64) -> Result<FarmInfo, Error<T>> {
        // Fetch farm next
        debug::info!("fetching farm");
        let resp_bytes = Self::fetch_farm_from_remote(farm_id).map_err(|e| {
            debug::error!("fetch_node_from_remote error: {:?}", e);
            <Error<T>>::HttpFetchingError
        })?;
    
        let resp_str = str::from_utf8(&resp_bytes).map_err(|err| {
            debug::info!("{:?}", err); 
            <Error<T>>::HttpFetchingError
        })?;
        debug::info!("{}", resp_str);
    
        let farm_info: FarmInfo = serde_json::from_str(&resp_str).map_err(|err| {
            debug::info!("{:?}", err); 
            <Error<T>>::HttpFetchingError
        })?;
        debug::info!("got farm response");
    
        Ok(farm_info)
    }
    
    /// This function uses the `offchain::http` API to query the remote github information,
	///   and returns the JSON response as vector of bytes.
	fn fetch_node_from_remote(node_id: Vec<u8>) -> Result<Vec<u8>, Error<T>> {        
        let mut h = EXPLORER_NODES.as_bytes().to_vec();
        h.extend_from_slice(&node_id);
    
        let p = str::from_utf8(&h).unwrap();

        debug::info!("sending request to: {:?}", p);

		let request = rt_offchain::http::Request::get(&p);

		let timeout = sp_io::offchain::timestamp()
			.add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

		let pending = request
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError);
		}

		Ok(response.body().collect::<Vec<u8>>())
    }

	fn fetch_farm_from_remote(farm_id: u64) -> Result<Vec<u8>, Error<T>> {        
        let mut h = EXPLORER_FARMS.as_bytes().to_vec();
        h.extend_from_slice(&Self::to_str_bytes(farm_id));
    
        let p = str::from_utf8(&h).unwrap();

        debug::info!("sending request to: {:?}", p);

		let request = rt_offchain::http::Request::get(&p);

		let timeout = sp_io::offchain::timestamp()
			.add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

		let pending = request
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError);
		}

		Ok(response.body().collect::<Vec<u8>>())
    }
    
    fn to_str_bytes(mut number: u64) -> Vec<u8> {
        let mut out = Vec::new();
    
        let mut l = true;
    
        while l {
            l = number > 9;
            let last_digit = number % 10;
            match last_digit {
                0 => out.extend_from_slice("0".as_bytes()),
                1 => out.extend_from_slice("1".as_bytes()),
                2 => out.extend_from_slice("2".as_bytes()),
                3 => out.extend_from_slice("3".as_bytes()),
                4 => out.extend_from_slice("4".as_bytes()),
                5 => out.extend_from_slice("5".as_bytes()),
                6 => out.extend_from_slice("6".as_bytes()),
                7 => out.extend_from_slice("7".as_bytes()),
                8 => out.extend_from_slice("8".as_bytes()),
                9 => out.extend_from_slice("9".as_bytes()),
                _ => unreachable!(),
            }
            number /= 10;
        }
    
        out.reverse();
        out
    }
}

impl<T: Trait> rt_offchain::storage_lock::BlockNumberProvider for Module<T> {
	type BlockNumber = T::BlockNumber;
	fn current_block_number() -> Self::BlockNumber {
	  <frame_system::Module<T>>::block_number()
    }
}