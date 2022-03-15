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
        ArticleIndexUpdated(T::AccountId, Vec<u8>, Vec<u8>),
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
   
    // the article post increasing counter, default 3
    #[pallet::storage]
    pub(super) type ArticleNonce = StorageValue<_, u64, ValueQuery>;
    // #[pallet::type_value]
    // pub(super) fn ArticleNonceDefault() -> u64 { 0 }
    // pub(super) type ArticleNonce =
    //    StorageValue<Value = u64, QueryKind = ValueQuery, OnEmpty = ArticleNonceDefault>;

    #[pallet::storage]
    pub(super) type CallCounterMap = 
        StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;  
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            // get article nonce
            // let nonce = ArticleNonce::get();

            // TODO: define the Handler, using Fn trait, just like this
            let method_table: Vec<(&[u8], dyn Fn(Json) -> Result<(), ()>)> = vec![
                (b"article_post", article::article_post),
                //b"article_update",
                //b"article_delete",
                //b"comment_post",
                //b"comment_delete",
            ];

            
            for (method_name, method_handler) in method_table {
                // get bottom number
                let bottom_key = Self::derive_pcbottom(&method_name);
                let stbottom = StorageValueRef::persistent(bottom_key);
                let bottom = if let Ok(Some(res)) = stbottom.get::<u64>() {
                    //log::info!("cached result: {:?}", res);
                    res
                }
                else {
                    stbottom.set(&0);
                    0
                }

                // get top number
                let top_key = Self::derive_pctop(&method_name);
                let sttop = StorageValueRef::persistent(top_key);
                let top = if let Ok(Some(res)) = sttop.get::<u64>() {
                    //log::info!("cached result: {:?}", res);
                    res
                }
                else {
                    sttop.set(&0);
                    0
                }

                // iterate on all incoming calls
                for n in bottom..top {
                    //let nonce = CallCounterMap::get(&method_name);
                    let storage_key = Self::derive_key(&method_name, n + 1);
                    let storage_ref = StorageValueRef::persistent(&storage_key);
                    
                    // get the value of this local storage key
                    if let Ok(Some(data)) = storage_ref.get::<Vec<u8>>() {
                        // data is the raw value, do something
                        
                        // decode the raw data, which is json format
                        // deserialized by serde
                        // let json_param = data.parse()
                        // extract the inner obj info
                        // let json_obj = json_param.obj

                        // put this json struct to the handler, call handler
                        // let result = *method_handler(json_obj)

                        // How to process the result of a handler?


                    }

                }
    
            }

            Ok(())
        }
    }

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
            // ArticleNonce::mutate(|nonce| nonce + 1);

            let method_name = b"article_post";
            if CallCounterMap::contains_key(&method_name) {
                CallCounterMap::mutate(&method_name, |top| {
                    top + 1
                })
            }
            else {
                // this first time call
                CallCounterMap::set(&method_name, 1);
            }
            // or just use mutate?
            // CallCounterMap::mutate(&method_name, |top| top + 1);
        

            // get latest nonce and write to local storage
            //let nonce = ArticleNonce::get();
            let nonce = CallCounterMap::get(&method_name);
            let storage_key = Self::derive_key(&method_name, nonce);
            offchain_index::set(&storage_key, &new_article_objbin);
            let storage_key = Self::derive_pctop(&method_name);
            offchain_index::set(&storage_key, &nonce.to_string());

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn article_update_index(
            origin: OriginFor<T>,
            article_id: Vec<u8>,
            article_hash: Vec<u8>,
            ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            //let sender = ensure_root(origin)?;

            // insert id hash pair index, if exists, override it
            ArticleIdHashMap::insert(&article_id, &article_hash);

            // Emit an event
            Self::deposit_event(Event::ArticleIndexUpdated(sender, article_id, article_hash));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn derive_key(method_name: &[u8], nonce: u64) -> Vec<u8> {
            WRITE_KEY_PREFIX.clone().into_iter()
                .chain(b":".into_iter())
                .chain(method_name.into_iter())
                .chain(b":".into_iter())
                .chain(nonce.to_string().into_iter())
                .copied()
                .collect::<Vec<u8>>()
        }

        fn derive_pctop(method_name: &[u8]) -> Vec<u8> {
            WRITE_KEY_PREFIX.clone().into_iter()
                .chain(b":".into_iter())
                .chain(method_name.into_iter())
                .chain(b":top".into_iter())
                .copied()
                .collect::<Vec<u8>>()
        }

        fn derive_pcbottom(method_name: &[u8]) -> Vec<u8> {
            WRITE_KEY_PREFIX.clone().into_iter()
                .chain(b":".into_iter())
                .chain(method_name.into_iter())
                .chain(b":bottom".into_iter())
                .copied()
                .collect::<Vec<u8>>()
        }
    }
}


