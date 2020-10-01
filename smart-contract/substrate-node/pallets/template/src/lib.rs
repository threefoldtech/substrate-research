#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Get,
};
use frame_system::ensure_signed;
use sp_core::RuntimeDebug;
use sp_std::{prelude::*, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct VolumeType {
    pub disk_type: u8,
    pub size: u64,
}

pub type NodeID = std::string::String;
pub type ID = u64;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub ReservationState get(fn reservation_state): map hasher(blake2_128_concat) ID => WorkloadState;
        pub ReservationsForAccount get(fn reservations_for_account): map hasher(blake2_128_concat) T::AccountId => Vec<ID>;
        pub WorkloadCreated get(fn workloads_created): map hasher (blake2_128_concat) NodeID => Vec<ID>;
        pub WorkloadDeployed get(fn workloads_deployed): map hasher (blake2_128_concat) NodeID => Vec<ID>;
        pub VolumeReservations get(fn volume_reservations): map hasher (blake2_128_concat) ID => VolumeType;
        ReservationID: u64;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// A new kyc proof has been added to the did belonging to the account, supplied by the
        /// given provider. [who, provider]
        // KycProofAdded(AccountId, Vec<u8>),
        ContractAdded(AccountId, NodeID),
        ContractRejected(u64),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// A kyc object from this provider already exists for this user.
        // ProviderExists,
        /// The provider name exceeds the maximum length.
        // ProviderTooLong,
        ReservationExists,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn create_contract(origin, node_id: std::string::String, volume: VolumeType) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            let reservation_id = ReservationID::get();

            VolumeReservations::insert(reservation_id, &volume);
            ReservationID::put(reservation_id + 1);

            ReservationsForAccount::<T>::mutate(&who, |list|  list.push(reservation_id));

            Self::deposit_event(RawEvent::ContractAdded(who, node_id));

            Ok(())
        }

        // Reject will be called by the farmer
        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn reject(origin, reservation_id: u64) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            VolumeReservations::remove(reservation_id);

            // Remove reservation for a user's account

            Self::deposit_event(RawEvent::ContractRejected(reservation_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn set_price(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn pay(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn claim_funds(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn set_result(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn cancel(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            
            Ok(())
        }
    }
}
