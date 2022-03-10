#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_io::offchain_index;
use sp_runtime::{offchain::storage::StorageValueRef, RuntimeDebug};

mod model;
mod article;

use model::{
    Article,
};

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

const WRITE_KEY_PREFIX: &[u8] = b"WRITEP";


#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }    
    
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ArticleIndexInserted(T::AccountId, Vec<u8>),
        ArticleIndexUpdated(T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ArticleIndexInsertError,
        ArticleIndexUpdateError,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // a pair of article_id vec<u8> -> article_item hash vec<u8>
    #[pallet::storage]
    pub(super) type ArticleIdHashMap = 
        StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;  
   
    // the article post increasing counter, default 0
    #[pallet::storage]
    pub(super) type ArticleNonce = StorageValue<_, u64, ValueQuery>;
    // #[pallet::type_value]
    // pub(super) fn ArticleNonceDefault() -> u64 { 0 }
    // pub(super) type ArticleNonce =
    //    StorageValue<Value = u64, QueryKind = ValueQuery, OnEmpty = ArticleNonceDefault>;
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(1_000)]
        pub fn article_post(
            origin: OriginFor<T>,
            new_article_objbin: Vec<u8>,
            ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            //let sender = ensure_root(origin)?;

            // self increasing
            ArticleNonce::mutate(|nonce| nonce + 1);

            // get latest nonce and write to local storage
            let nonce = ArticleNonce::get();
            let storage_key = WRITE_KEY_PREFIX.to_string() + ":" + "article_post:" + &nonce.to_string();
            offchain_index::set(&storage_key, &new_article_objbin);
            let storage_key = WRITE_KEY_PREFIX.to_string() + ":" + "article_post:top";
            offchain_index::set(&storage_key, &nonce.to_string());

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            proof: Vec<u8>,
            ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&proof);

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&proof);

            // Emit an event that the claim was erased.
            Self::deposit_event(Event::ClaimRevoked(sender, proof));
            Ok(())
        }
    }
}


