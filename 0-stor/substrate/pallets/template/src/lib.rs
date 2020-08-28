#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, Hashable,
};
use frame_system::{self as system, ensure_signed};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    // Add other types and constants required to configure this pallet.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    // It is important to update your storage name so that your pallet's
    // storage items are isolated from other pallets.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as TemplateModule {
        /// Storage backend for 0-stor.
        ///
        /// Key is the blake2b_128 hash of the namespace + plain key bytes
        /// Item is a tuple, first element is the encoded metadata, second item is the account
        /// ID which originally inserted the metadata. Only this account can later update or delete
        /// it.
        MetaStor get(fn meta_stor): map hasher(identity) Vec<u8> => (Vec<u8>, T::AccountId);
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// New metadata was inserted in a namespace. [key, who]
        MetadataCreated(AccountId, Vec<u8>),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Tried to insert metadata in a namespace with a key which already exists in said
        /// namespace.
        MetadataExists,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing errors
        // this includes information about your errors in the node's metadata.
        // it is needed only if you are using errors in your pallet
        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        /// Set metadata in a given namespace with a given key. Ownership of the metadata is
        /// assigned to the caller of the function. It is an error to set metadata on a key if that
        /// key is already in use in the namespace, even if the previous key is owned by the
        /// calling user.
        #[weight = 10_000] // TODO
        pub fn set_metadata(origin, namespace: Vec<u8>, key: Vec<u8>, metadata: Vec<u8>) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;

            // construct key
            let namespace_hash = &namespace.blake2_128();
            let mut hashed_key: Vec<u8>= namespace_hash[..].into();
            hashed_key.extend_from_slice(&key);
            // hashed_key is now namespace_hash + raw key bytes

            // Makse sure this is not an update, there is a separate function for that
            ensure!(!MetaStor::<T>::contains_key(&hashed_key), Error::<T>::MetadataExists);

            MetaStor::<T>::insert(&hashed_key, (&metadata, &sender));

            Self::deposit_event(RawEvent::MetadataCreated(sender, key));

            Ok(())
        }
    }
}
