#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::{DispatchError, DispatchResult}, ensure, debug,
    traits::{
		Currency, Get, ExistenceRequirement::AllowDeath, Randomness
    },
    sp_runtime::{traits::AccountIdConversion, ModuleId}
};
use frame_system::{self as system, ensure_signed};
use sp_core::{RuntimeDebug, H256};
use sp_std::{prelude::*, vec::Vec};

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

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type RandomnessSource: Randomness<H256>;
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
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        ReservationExists,
        ContractExists,
        ContractNotExists,
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
        pub fn pay(origin, reservation_id: u64, amount: BalanceOf<T>) -> DispatchResult {
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

        // #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        // pub fn claim_funds(origin) -> DispatchResult {
        //     let who = ensure_signed(origin)?;
            
        //     Ok(())
        // }

        // #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        // pub fn set_result(origin) -> DispatchResult {
        //     let who = ensure_signed(origin)?;
            
        //     Ok(())
        // }

        // #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        // pub fn cancel(origin) -> DispatchResult {
        //     let who = ensure_signed(origin)?;
            
        //     Ok(())
        // }
    }
}
