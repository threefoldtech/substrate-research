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
        traits::AccountIdConversion, ModuleId, traits::SaturatedConversion,
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
use sp_core::{RuntimeDebug, H256, ed25519};
use sp_std::{
	prelude::*, str
};
use alt_serde::{Deserialize, Deserializer};
use hex::FromHex;
use sp_core::crypto::KeyTypeId;
use bs58;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
use pallet_timestamp as timestamp;
use fixed::{types::I32F32, types::U128F0, types::U64F64};

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

pub struct RSU {
    cru: u64,
    hru: I32F32,
    sru: I32F32,
    mru: I32F32
}

impl VolumeType {
    fn get_rsu(&self) -> RSU {
        match self.disk_type {
            1 => {
                RSU{
                    hru: I32F32::from_num(self.size),
                    sru: I32F32::from_num(0),
                    mru: I32F32::from_num(0),
                    cru: 0,
                }
            }
            2 => {
                RSU{
                    hru: I32F32::from_num(0),
                    sru: I32F32::from_num(self.size),
                    mru: I32F32::from_num(0),
                    cru: 0,
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct Contract<T: Trait> {
    resource_prices: ResourcePrice,
    account_id: T::AccountId,
    node_id: Vec<u8>,
    farmer_account: T::AccountId,
    user_account: T::AccountId,
    accepted: bool,
    workload_state: WorkloadState,
    expires_at: u64,
    last_claimed: u64
}

impl<T> Default for Contract<T>
    where T: Trait
{
    fn default() -> Contract<T> {
        let account_id = PALLET_ID.into_account();
        let farmer_account = PALLET_ID.into_account();
        let user_account = PALLET_ID.into_account();

        Contract {
            resource_prices: ResourcePrice::default(),
            account_id,
            node_id: [0].to_vec(),
            farmer_account,
            user_account,
            accepted: false,
            workload_state: WorkloadState::Created,
            expires_at: 0,
            last_claimed: 0
        }
    }
}

pub const EXPLORER_NODES: &str = "https://explorer.devnet.grid.tf/explorer/nodes/";
pub const EXPLORER_FARMS: &str = "https://explorer.devnet.grid.tf/explorer/farms/";
pub const EXPLORER_USERS: &str = "https://explorer.devnet.grid.tf/explorer/users/";
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
    resource_prices: Vec<ResourcePrice>
}

struct Farm {
    farm_info: FarmInfo,
    pubkey: Vec<u8>,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct UserInfo {
    #[serde(deserialize_with = "de_string_to_bytes")]
    pubkey: Vec<u8>
}

#[serde(crate = "alt_serde")]
#[derive(PartialEq, Eq, PartialOrd, Ord, Deserialize, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct ResourcePrice {
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

pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type RandomnessSource: Randomness<H256>;
    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub ReservationState get(fn reservation_state): map hasher(blake2_128_concat) u64 => WorkloadState;
        pub ReservationsForAccount get(fn reservations_for_account): map hasher(blake2_128_concat) T::AccountId => Vec<u64>;
        pub VolumeReservations get(fn volume_reservations): map hasher (blake2_128_concat) u64 => VolumeType;
        pub Contracts get(fn contracts): map hasher (blake2_128_concat) u64 => Contract<T>;
        pub ContractPerExpiration get(fn contracts_per_expiration): map hasher (blake2_128_concat) u64 => Vec<u64>;
        ReservationID: u64;
        LastBlockTime: u64;
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
        ContractDeployed(Vec<u8>, u64),
        ContractCancelled(Vec<u8>, u64),
        // Will signal a contract being accepted for a NodeID and a reservation ID
        ContractAccepted(Vec<u8>, u64),
        ContractFundsClaimed(u64),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        ReservationExists,
        ContractExists,
        ContractNotExists,
        ContractNotAccepted,
        ContractNotDeployed,
        UnknownOffchainMux,
        HttpFetchingError,
        // Error returned when making signed transactions in off-chain worker
		NoLocalAcctForSigning,
        OffchainSignedTxError,
        NoLocalAcctForSignedTx,
        UnauthorizedFarmer,
        UnauthorizedUser,
        UnauthorizedNode,
        NotEnoughBalanceToClaim,
        ClaimError,
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

            // Add the user account to the contract
            contract.user_account = who.clone();

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
        pub fn pay(origin, reservation_id: u64, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let mut contract = Contracts::<T>::get(reservation_id);

            // ensure!(contract.accepted == true, Error::<T>::ContractNotAccepted);

            debug::info!("Transfering: {:?} from {:?} to contract accountId: {:?}", &amount, &who, &contract.account_id);
            // Transfer currency to the contracts account
            T::Currency::transfer(&who, &contract.account_id, amount, AllowDeath)
                .map_err(|_| DispatchError::Other("Can't make transfer"))?;

            // Reevauluate contract expiration date
            // check if the expiration date is set first in order to not confuse the user
            let volume = VolumeReservations::get(reservation_id);

            let sru = volume.get_rsu();

            let price_per_hour = Self::get_price_per_hour(&contract.resource_prices, sru);
            let price_per_sec = U64F64::from_num(price_per_hour) * U64F64::from_num(1e12) / U64F64::from_num(3600);

            // Get the contract's balance
            let balance: BalanceOf<T> = T::Currency::free_balance(&contract.account_id);
            let balances_as_u128: u128 = balance.saturated_into::<u128>();

            let expires_at = (U128F0::from_num(balances_as_u128) / U128F0::from_num(price_per_sec)).to_num::<u64>();

            // Update expires_at if there is an expiration date, this means the user is probably re-funding the contract
            if contract.expires_at > 0 {
                // Since the contract is expiration date will be updated we need to remove it from the list first
                // in order to prevent it from getting cancelled before the new expiration date
                ContractPerExpiration::mutate(&contract.expires_at, |list|  {
                    debug::info!("list: {:?}", list);
                    if list.len() > 0 {
                        let index = list.iter().position(|x| x == &reservation_id).unwrap();
                        list.remove(index);
                    }
                });
                
                contract.expires_at += expires_at;
                debug::info!("Reevauluating contract expiration, expires at: {:?}", &contract.expires_at);
            } else {
                let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
                contract.expires_at = now + expires_at;
                debug::info!("Contract will expire at: {:?}", &contract.expires_at);
            }
            
            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);
            // Insert the reservationID at contract expiration date
            ContractPerExpiration::mutate(&contract.expires_at, |list|  list.push(reservation_id));

            Self::deposit_event(RawEvent::ContractPaid(contract.account_id, reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn set_contract_price(origin, reservation_id: u64, resource_prices: ResourcePrice, farmer_account: <T as frame_system::Trait>::AccountId) -> DispatchResult {
            // TODO: Only off chain worker can sign this
            let _ = ensure_signed(origin)?;

            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let mut contract = Contracts::<T>::get(reservation_id);

            contract.resource_prices = resource_prices;
            contract.farmer_account = farmer_account;

            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);
            
            Self::deposit_event(RawEvent::ContractUpdated(contract.account_id, reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn accept_contract(origin, reservation_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let mut contract = Contracts::<T>::get(reservation_id);

            // Ensure only the farmer of the contract can accept the contract
            ensure!(contract.farmer_account == who, Error::<T>::UnauthorizedFarmer);

            contract.accepted = true;

            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);
            
            Self::deposit_event(RawEvent::ContractAccepted(contract.node_id, reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn claim_funds(origin, reservation_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let mut contract = Contracts::<T>::get(reservation_id);

            ensure!(contract.workload_state == WorkloadState::Deployed, Error::<T>::ContractNotDeployed);

            // Ensure only the farmer of the contract can claim the funds
            ensure!(contract.farmer_account == who, Error::<T>::UnauthorizedFarmer);

            // Get the contract's balance
            let mut balance: BalanceOf<T> = T::Currency::free_balance(&contract.account_id);

            // Evaluate if the farmer can claim funds, if yes, calculate how much
            let volume = VolumeReservations::get(reservation_id);

            let sru = volume.get_rsu();

            let price_per_hour = Self::get_price_per_hour(&contract.resource_prices, sru);
            let price_per_sec = U64F64::from_num(price_per_hour) * U64F64::from_num(1e12) / U64F64::from_num(3600);

            let now = <timestamp::Module<T>>::get().saturated_into::<u64>();

            // convert to seconds
            let diff = (now - contract.last_claimed) / 1000;
            ensure!(diff > 0, Error::<T>::ClaimError);

            debug::info!("{:?} seconds have passed since last claimed, price per sec: {:?}", diff, price_per_sec);

            let amount_to_claim = (U64F64::from_num(diff) * price_per_sec).to_num::<u128>();
            let balance_as_u128 = balance.saturated_into::<u128>();

            debug::info!("Trying to claim {:?}, from contract with balance: {:?}", &amount_to_claim, &balance_as_u128);

            if amount_to_claim <= balance_as_u128 {
                balance = amount_to_claim.saturated_into();
            }

            debug::info!("Transfering: {:?} from contract {:?} to farmer {:?}", &balance, &contract.account_id, &who);
            // Transfer currency to the farmers account
            T::Currency::transfer(&contract.account_id, &contract.farmer_account, balance, AllowDeath)
                .map_err(|_| DispatchError::Other("Can't make transfer"))?;

            contract.last_claimed = now;

            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);

            Self::deposit_event(RawEvent::ContractFundsClaimed(reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn cancel_contract(origin, reservation_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let contract = Contracts::<T>::get(reservation_id);

            // Ensure only the farmer of the contract can accept the contract
            ensure!(contract.user_account == who, Error::<T>::UnauthorizedUser);

            // Get the contract's balance
            let balance: BalanceOf<T> = T::Currency::free_balance(&contract.account_id);

            debug::info!("Transfering: {:?} from contract {:?} to user {:?}", &balance, &contract.account_id, &who);
            // Transfer currency to the users account
            T::Currency::transfer(&contract.account_id, &contract.user_account, balance, AllowDeath)
                .map_err(|_| DispatchError::Other("Can't make transfer"))?;

            Self::deposit_event(RawEvent::ContractCancelled(contract.node_id, reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn contract_cancelled(origin, reservation_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let mut contract = Contracts::<T>::get(reservation_id);

            // Ensure the node signed
            let mut decoded = [0u8;32];
            let _ = bs58::decode(&contract.node_id).into(&mut decoded).unwrap();

            let node_address = ed25519::Public::from_raw(decoded);
            ensure!(T::AccountId::decode(&mut &node_address[..]).unwrap_or_default() == who, Error::<T>::UnauthorizedNode);

            contract.workload_state = WorkloadState::Cancelled;

            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);
            
            Self::deposit_event(RawEvent::ContractUpdated(contract.account_id, reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn contract_deployed(origin, reservation_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Contracts::<T>::contains_key(&reservation_id), Error::<T>::ContractNotExists);

            let mut contract = Contracts::<T>::get(reservation_id);

            // Ensure the node signed
            let mut decoded = [0u8;32];
            let _ = bs58::decode(&contract.node_id).into(&mut decoded).unwrap();

            let node_address = ed25519::Public::from_raw(decoded);
            ensure!(T::AccountId::decode(&mut &node_address[..]).unwrap_or_default() == who, Error::<T>::UnauthorizedNode);

            contract.workload_state = WorkloadState::Deployed;

            let volume = VolumeReservations::get(reservation_id);

            let sru = volume.get_rsu();

            let price_per_hour = Self::get_price_per_hour(&contract.resource_prices, sru);
            let price_per_sec = U64F64::from_num(price_per_hour) * U64F64::from_num(1e12) / U64F64::from_num(3600);

            // Get the contract's balance
            let balance: BalanceOf<T> = T::Currency::free_balance(&contract.account_id);
            let balances_as_u128: u128 = balance.saturated_into::<u128>();

            let expires_at = (U128F0::from_num(balances_as_u128) / U128F0::from_num(price_per_sec)).to_num::<u64>();

            // Update expires at
            // Calculate based on farmer prices
            let now = <timestamp::Module<T>>::get().saturated_into::<u64>();
            let then = (now / 1000) + expires_at;
            contract.expires_at = then;
            // Set last claimed in order to know when to contract was deployed
            contract.last_claimed = now;
            debug::info!("expires at: {:?}", then);

            // Update the contract
            Contracts::<T>::insert(&reservation_id, &contract);
            
            Self::deposit_event(RawEvent::ContractDeployed(contract.node_id, reservation_id));

            Ok(())
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            debug::info!("Entering off-chain worker");
    
            let _ = Self::offchain_signed_tx(block_number);
        }

        fn on_finalize(b: T::BlockNumber) {
            let block_number_as_u64: u64 = b.try_into().unwrap_or(0) as u64;
            debug::info!("Entering on_finalize, block number: {:?}", block_number_as_u64);
            let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
            if block_number_as_u64 <= 1 {
                LastBlockTime::put(now);
                return
            }

            let last_block_time = LastBlockTime::get();
            debug::info!("last block time: {:?}", last_block_time);

            if last_block_time == 0 {
                LastBlockTime::put(now);
                return
            }

            for time in last_block_time..now {
                // Get reservationID at a specific timestamp
                let reservation_ids = ContractPerExpiration::get(time);
                for reservation_id in reservation_ids {
                    match Self::decomission_contract(reservation_id, time) {
                        Ok(()) => {
                            debug::info!("decomission of contract: {:?} success", reservation_id)
                        }
                        Err(err) => { debug::info!("error occured: {:?}", err); }
                    }
                }
            }
            
            LastBlockTime::put(now);
        }
    }
}   

impl<T: Trait> Module<T> {
    fn decomission_contract(reservation_id: u64, time: u64) -> Result<(), DispatchError> {
        let mut contract = Contracts::<T>::get(reservation_id);
        debug::info!("contract: {:?} for reservation ID {:?} found at time: {:?}", contract.account_id, reservation_id, time);

        // Get the contract's balance
        let balance: BalanceOf<T> = T::Currency::free_balance(&contract.account_id);

        debug::info!("Transfering {:?} from contract: {:?} to farmer: {:?}", &balance, &contract.account_id, &contract.farmer_account);
        // Transfer currency to the users account
        T::Currency::transfer(&contract.account_id, &contract.farmer_account, balance, AllowDeath).map_err(|err| {
            debug::info!("{:?}", err); 
            err
        })?;

        contract.workload_state = WorkloadState::Cancelled;

        // Update the contract
        Contracts::<T>::insert(&reservation_id, &contract);

        ContractPerExpiration::mutate(&contract.expires_at, |list|  {
            debug::info!("list: {:?}", list);
            if list.len() > 0 {
                let index = list.iter().position(|x| x == &reservation_id).unwrap();
                list.remove(index);
            }
        });

        Self::deposit_event(RawEvent::ContractCancelled(contract.node_id, reservation_id));

        Ok(())
    }


    fn get_price_per_hour(resource_prices: &ResourcePrice, rsu: RSU) -> I32F32 {
        let cru = resource_prices.cru * rsu.cru;
        let hru = I32F32::from_num(resource_prices.hru) * rsu.hru;
        let sru = I32F32::from_num(resource_prices.sru) * rsu.sru;
        let mru = I32F32::from_num(resource_prices.mru) * rsu.mru;

        I32F32::from_num(cru) + hru + sru + mru
    }


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

        
        let farm = Self::fetch_farmer_prices(contract.node_id).map_err(|err| {
            debug::info!("{:?}", err); 
            err
        })?;
        
        reservation_id_storage.set(&reservation_id);

        debug::info!("Pubkey before parsing hex farm: {:?}, {:?}", farm.farm_info.id, farm.pubkey);

        let decoded = <[u8; 32]>::from_hex(farm.pubkey.clone()).expect("Decoding failed");
        let farmer_address = ed25519::Public::from_raw(decoded);

        // retrieve contract account
        let signer = Signer::<T, T::AuthorityId>::any_account();

        let result = signer.send_signed_transaction(|_acct|
            Call::set_contract_price(reservation_id, farm.farm_info.resource_prices[0].clone(), T::AccountId::decode(&mut &farmer_address[..]).unwrap_or_default())
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

	fn fetch_farmer_prices(node_id: Vec<u8>) -> Result<Farm, Error<T>> {
		let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
			b"offchain-explorer::lock", LOCK_BLOCK_EXPIRATION,
			rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION)
		);
        
        if let Ok(_guard) = lock.try_lock() {
			match Self::fetch_n_parse_node(node_id) {
				Ok(node_info) => {
                    match Self::fetch_n_parse_farm(node_info.farm_id) {
                        Ok(farm_info) => {
                            match Self::fetch_n_parse_user(farm_info.threebot_id) {
                                Ok(user_info) => {
                                    let farm = Farm {
                                        farm_info,
                                        pubkey: user_info.pubkey
                                    };
                                    return Ok(farm)
                                }
                                Err(err) => { return Err(err); }
                            }
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
        // debug::info!("{}", resp_str);

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
        // debug::info!("{}", resp_str);
    
        let farm_info: FarmInfo = serde_json::from_str(&resp_str).map_err(|err| {
            debug::info!("{:?}", err); 
            <Error<T>>::HttpFetchingError
        })?;
        debug::info!("got farm response");
    
        Ok(farm_info)
    }

    fn fetch_n_parse_user(threebot_id: u64) -> Result<UserInfo, Error<T>> {
        debug::info!("fetching user");
        let resp_bytes = Self::fetch_user_from_remote(threebot_id).map_err(|e| {
            debug::error!("fetch_node_from_remote error: {:?}", e);
            <Error<T>>::HttpFetchingError
        })?;
    
        let resp_str = str::from_utf8(&resp_bytes).map_err(|err| {
            debug::info!("{:?}", err); 
            <Error<T>>::HttpFetchingError
        })?;
        // debug::info!("{}", resp_str);
    
        let user_info: UserInfo = serde_json::from_str(&resp_str).map_err(|err| {
            debug::info!("{:?}", err); 
            <Error<T>>::HttpFetchingError
        })?;
        debug::info!("got farm response");
    
        Ok(user_info)
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

    fn fetch_user_from_remote(threebot_id: u64) -> Result<Vec<u8>, Error<T>> {        
        let mut h = EXPLORER_USERS.as_bytes().to_vec();
        h.extend_from_slice(&Self::to_str_bytes(threebot_id));

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