#![cfg_attr(not(feature = "std"),no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        //The maximum length of claim that can be added.
        #[pallet::constant]
        type MaxClaimLength: Get<u32>;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxClaimLength>,
        (T::AccountId, T::BlockNumber),
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config>{
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        ClaimTooLong,
        ClaimNotExist,
        NotClaimOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender: <T as Config>::AccountId = ensure_signed(origin)?;

            let bounded_claim: BoundedVec<u8, <T as Config>::MaxClaimLength> = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
                .map_err(op:|_| Error::<T>::ClaimTooLong)?;
            ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist)

            Proofs::<T>::insert(
                key:&bounded_claim,
                val:(sender.clone(), frame_system::Pallet::<T>::block_number()),
            );

            Self::deposit_event(Event::ClaimCreated(sender, claim));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender: <T as Config>::AccountId = ensure_signed(origin)?;

            let bounded_claim: BoundedVec<u8, <T as Config>::MaxClaimLength> = 
                BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
                .map_err(op:|_| Error::<T>::ClaimTooLong)?;
            let (owner: <T as Config>::AccountId, _) = Proofs::<T>::get(key:&bounded_claim).ok_or(err: Error::<T>::ClaimNotExist)?;

            ensure!(owner == sender, Error::<T>::NotClaimOwner);

            Proofs::<T>::remove(key:&bounded_claim);

            Self::deposit_event(Event::ClaimRevoked(sender, claim));

            Ok(().into())
        }

        #[pallet::weight(0)]
        #[pallet::call_index(3)]
        pub fn transfer_claim(
	        origin: OriginFor<T>, 
            claim: T::Hash, 
            receiver: T::AccountId
        ) -> DispatchResult {
           // Check that the extrinsic was signed and get the signer.
        	// This function will return an error if the extrinsic is not signed.
            let sender = ensure_signed(origin)?;
    
             // Get owner of the claim, if none return an error.
            let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            // Get the block number from the FRAME System pallet.
            let current_block = <frame_system::Pallet<T>>::block_number();

            // Remove claim from storage.
            Claims::<T>::remove(&claim);

            // Store the claim with the sender and block number.
            Claims::<T>::insert(&claim, (&receiver, current_block));

            // Emit an event that the claim was erased.
            Self::deposit_event(Event::ClaimTransfered { who: sender, claim, receiver });

            Ok(())
        }
    }
}mod pallet