// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Aura Module
//!
//! - [`Config`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The Aura module extends Aura consensus by managing offline reporting.
//!
//! ## Interface
//!
//! ### Public Functions
//!
//! - `slot_duration` - Determine the Aura slot-duration based on the Timestamp module
//!   configuration.
//!
//! ## Related Modules
//!
//! - [Timestamp](../pallet_timestamp/index.html): The Timestamp module is used in Aura to track
//! consensus rounds (via `slots`).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::manual_inspect)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    dispatch::DispatchResult,
    traits::{ConstU32, DisabledValidators, FindAuthor, Get, OnTimestampSet, OneSessionHandler},
    BoundedSlice, BoundedVec, ConsensusEngineId, Parameter,
};
use log;
use sp_consensus_aura::{AuthorityIndex, ConsensusLog, Slot, AURA_ENGINE_ID};
use sp_runtime::{
    generic::DigestItem,
    traits::{IsMember, Member, SaturatedConversion, Saturating, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
    RuntimeAppPublic,
};

pub mod migrations;
mod mock;
mod tests;

pub use pallet::*;

const LOG_TARGET: &str = "runtime::aura";

/// A slot duration provider which infers the slot duration from the
/// [`pallet_timestamp::Config::MinimumPeriod`] by multiplying it by two, to ensure
/// that authors have the majority of their slot to author within.
///
/// This was the default behavior of the Aura pallet and may be used for
/// backwards compatibility.
pub struct MinimumPeriodTimesTwo<T>(core::marker::PhantomData<T>);

impl<T: pallet_timestamp::Config> Get<T::Moment> for MinimumPeriodTimesTwo<T> {
    fn get() -> T::Moment {
        <T as pallet_timestamp::Config>::MinimumPeriod::get().saturating_mul(2u32.into())
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: pallet_timestamp::Config + frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The identifier type for an authority.
        type AuthorityId: Member
            + Parameter
            + RuntimeAppPublic
            + MaybeSerializeDeserialize
            + MaxEncodedLen;
        /// The maximum number of authorities that the pallet can hold.
        type MaxAuthorities: Get<u32>;

        /// A way to check whether a given validator is disabled and should not be authoring blocks.
        /// Blocks authored by a disabled validator will lead to a panic as part of this module's
        /// initialization.
        type DisabledValidators: DisabledValidators;

        /// Whether to allow block authors to create multiple blocks per slot.
        ///
        /// If this is `true`, the pallet will allow slots to stay the same across sequential
        /// blocks. If this is `false`, the pallet will require that subsequent blocks always have
        /// higher slots than previous ones.
        ///
        /// Regardless of the setting of this storage value, the pallet will always enforce the
        /// invariant that slots don't move backwards as the chain progresses.
        ///
        /// The typical value for this should be 'false' unless this pallet is being augmented by
        /// another pallet which enforces some limitation on the number of blocks authors can create
        /// using the same slot.
        type AllowMultipleBlocksPerSlot: Get<bool>;

        /// The slot duration Aura should run with, expressed in milliseconds.
        /// The effective value of this type should not change while the chain is running.
        ///
        /// For backwards compatibility either use [`MinimumPeriodTimesTwo`] or a const.
        #[pallet::constant]
        type SlotDuration: Get<<Self as pallet_timestamp::Config>::Moment>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(core::marker::PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            if let Err(e) = Self::check_license_and_halt_if_needed() {
                log::error!(
                    target: LOG_TARGET,
                    "Error in offchain worker at block {:?}: {:?}",
                    block_number,
                    e
                );
            }
        }

        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // Check if halt was requested by offchain worker
            use sp_runtime::offchain::storage::StorageValueRef;
            let storage_halt = StorageValueRef::persistent(b"licensed_aura::halt_requested");
            if let Some(true) = storage_halt.get::<bool>().unwrap_or(None) {
                if !HaltProduction::<T>::get() {
                    HaltProduction::<T>::put(true);
                    HaltedAtBlock::<T>::put(n);
                    let reason = b"License check failed by offchain worker".to_vec();
                    let bounded_reason =
                        BoundedVec::<u8, ConstU32<256>>::try_from(reason).unwrap_or_default();
                    HaltReason::<T>::put(bounded_reason);
                    StorageValueRef::persistent(b"licensed_aura::halt_requested").clear();
                }
            }

            // Check if block production is halted
            if HaltProduction::<T>::get() {
                // Optional: Auto-recovery after 100 blocks (this can be made configurable)
                if let Some(halted_at) = HaltedAtBlock::<T>::get() {
                    let blocks_halted = n.saturating_sub(halted_at);
                    // Auto-resume after 100 blocks
                    if blocks_halted > 100u32.into() {
                        HaltProduction::<T>::put(false);
                        HaltedAtBlock::<T>::kill();
                        HaltReason::<T>::kill();
                        log::info!(
                            target: LOG_TARGET,
                            "Auto-resuming block production after {:?} blocks",
                            blocks_halted
                        );
                    } else {
                        // Panic to invalidate the block
                        if let Some(reason_bytes) = HaltReason::<T>::get() {
                            if let Ok(reason_str) = core::str::from_utf8(&reason_bytes) {
                                panic!(
                                    "Block production halted at block {:?}. Reason: {}",
                                    halted_at, reason_str
                                );
                            } else {
                                panic!(
                                    "Block production halted at block {:?}. Reason: Invalid UTF-8",
                                    halted_at
                                );
                            }
                        } else {
                            panic!(
                                "Block production halted at block {:?}. Reason: No reason provided",
                                halted_at
                            );
                        }
                    }
                } else {
                    // First time halting, record the block number
                    HaltedAtBlock::<T>::put(n);
                    panic!("Block production halted at block {:?}", n);
                }
            }

            // Original AURA logic continues here...
            if let Some(new_slot) = Self::current_slot_from_digests() {
                let current_slot = CurrentSlot::<T>::get();

                if T::AllowMultipleBlocksPerSlot::get() {
                    assert!(current_slot <= new_slot, "Slot must not decrease");
                } else {
                    assert!(current_slot < new_slot, "Slot must increase");
                }

                CurrentSlot::<T>::put(new_slot);

                if let Some(n_authorities) = <Authorities<T>>::decode_len() {
                    let authority_index = *new_slot % n_authorities as u64;
                    if T::DisabledValidators::is_disabled(authority_index as u32) {
                        panic!(
							"Validator with index {:?} is disabled and should not be attempting to author blocks.",
							authority_index,
						);
                    }
                }

                // TODO [#3398] Generate offence report for all authorities that skipped their
                // slots.

                T::DbWeight::get().reads_writes(3, 2) // Updated: Added reads for HaltProduction check
            } else {
                T::DbWeight::get().reads(2) // Updated: Added read for HaltProduction check
            }
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(_: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            Self::do_try_state()
        }
    }

    /// The current authority set.
    #[pallet::storage]
    pub type Authorities<T: Config> =
        StorageValue<_, BoundedVec<T::AuthorityId, T::MaxAuthorities>, ValueQuery>;

    /// The current slot of this block.
    ///
    /// This will be set in `on_initialize`.
    #[pallet::storage]
    pub type CurrentSlot<T: Config> = StorageValue<_, Slot, ValueQuery>;

    /// Flag to halt block production.
    #[pallet::storage]
    pub type HaltProduction<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Block number when halt was triggered (for auto-recovery).
    #[pallet::storage]
    pub type HaltedAtBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// Optional: Store the reason for halting.
    #[pallet::storage]
    pub type HaltReason<T: Config> = StorageValue<_, BoundedVec<u8, ConstU32<256>>, OptionQuery>;

    /// License key for validation against the API.
    #[pallet::storage]
    pub type LicenseKey<T: Config> = StorageValue<_, BoundedVec<u8, ConstU32<128>>, OptionQuery>;

    /// Events for the pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Block production was halted.
        ProductionHalted { block_number: BlockNumberFor<T> },
        /// Block production was resumed.
        ProductionResumed { block_number: BlockNumberFor<T> },
    }

    /// Errors for the pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// Halt reason is too long.
        ReasonTooLong,
        /// License key is too long (max 128 bytes).
        LicenseKeyTooLong,
        /// License key is not set.
        LicenseKeyNotSet,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Halt block production (requires sudo or governance).
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().writes(3))]
        pub fn sudo_halt_production(
            origin: OriginFor<T>,
            reason: Option<Vec<u8>>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            Self::halt_production_internal(reason)?;
            Self::deposit_event(Event::ProductionHalted {
                block_number: current_block,
            });
            Ok(())
        }

        /// Resume block production (requires sudo or governance).
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().writes(3))]
        pub fn sudo_resume_production(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            Self::resume_production_internal();
            Self::deposit_event(Event::ProductionResumed {
                block_number: current_block,
            });
            Ok(())
        }

        /// Halt production from offchain worker (unsigned transaction).
        /// This is specifically for the offchain worker pallet to call when license check fails.
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().writes(3))]
        pub fn offchain_worker_halt_production(
            origin: OriginFor<T>,
            reason: Option<Vec<u8>>,
        ) -> DispatchResult {
            // This accepts unsigned transactions from the offchain worker
            ensure_none(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            Self::halt_production_internal(reason)?;
            Self::deposit_event(Event::ProductionHalted {
                block_number: current_block,
            });
            Ok(())
        }

        /// Set the license key for API validation (requires sudo or governance).
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn set_license_key(origin: OriginFor<T>, license_key: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;

            let bounded_key = BoundedVec::<u8, ConstU32<128>>::try_from(license_key)
                .map_err(|_| Error::<T>::LicenseKeyTooLong)?;
            LicenseKey::<T>::put(bounded_key);

            log::info!(target: LOG_TARGET, "License key updated");
            Ok(())
        }
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub authorities: Vec<T::AuthorityId>,
        #[cfg_attr(
            feature = "std",
            serde(
                default,
                serialize_with = "license_key_serde::serialize",
                deserialize_with = "license_key_serde::deserialize",
            )
        )]
        pub license_key: Option<Vec<u8>>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::initialize_authorities(&self.authorities);

            // Initialize license key if provided
            if let Some(ref key) = self.license_key {
                let bounded_key = BoundedVec::<u8, ConstU32<128>>::try_from(key.clone())
                    .expect("License key too long for genesis config");
                LicenseKey::<T>::put(bounded_key);
            }
        }
    }

    /// Allow the chainspec to keep the license key as a readable string while the runtime stores
    /// it as bytes. Only compiled for std (chainspec generation path).
    #[cfg(feature = "std")]
    mod license_key_serde {
        use super::*;
        use serde::{Deserialize, Deserializer, Serializer};

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Input {
            String(String),
            Bytes(Vec<u8>),
        }

        pub fn serialize<S>(key: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match key {
                Some(bytes) => serializer.serialize_str(&String::from_utf8_lossy(bytes)),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value: Option<Input> = Option::deserialize(deserializer)?;
            Ok(value.map(|v| match v {
                Input::String(s) => s.into_bytes(),
                Input::Bytes(b) => b,
            }))
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            match call {
                Call::offchain_worker_halt_production { reason: _ } => {
                    // Only allow one halt transaction per block
                    ValidTransaction::with_tag_prefix("AuraHalt")
                        .priority(u64::MAX) // High priority
                        .and_provides("halt_production")
                        .longevity(1) // Valid for 1 block
                        .propagate(true)
                        .build()
                }
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Internal function to halt block production.
    /// Can only be called through sudo or offchain worker extrinsics.
    fn halt_production_internal(reason: Option<Vec<u8>>) -> DispatchResult {
        HaltProduction::<T>::put(true);

        if let Some(r) = reason {
            let bounded_reason = BoundedVec::<u8, ConstU32<256>>::try_from(r)
                .map_err(|_| Error::<T>::ReasonTooLong)?;
            HaltReason::<T>::put(bounded_reason);
        }

        log::warn!(target: LOG_TARGET, "Block production halted!");
        Ok(())
    }

    /// Internal function to resume block production.
    /// Can only be called through sudo extrinsic.
    fn resume_production_internal() {
        HaltProduction::<T>::put(false);
        HaltedAtBlock::<T>::kill();
        HaltReason::<T>::kill();
        log::info!(target: LOG_TARGET, "Block production resumed!");
    }

    /// Check if production is halted (read-only).
    pub fn is_halted() -> bool {
        HaltProduction::<T>::get()
    }

    /// Check license validity and submit halt transaction if needed.
    fn check_license_and_halt_if_needed() -> Result<(), &'static str> {
        use sp_runtime::offchain::{http, storage::StorageValueRef, Duration};

        let storage = StorageValueRef::persistent(b"licensed_aura::last_check");
        let now = sp_io::offchain::timestamp();

        let last_check = storage.get::<u64>().unwrap_or(None).unwrap_or(0);
        // Check every 30 seconds
        if now.unix_millis() - last_check < 30000 {
            return Ok(());
        }

        // Get the license key from storage
        let license_key_bytes = LicenseKey::<T>::get().ok_or("License key not set")?;
        let license_key =
            alloc::str::from_utf8(&license_key_bytes).map_err(|_| "Invalid license key UTF8")?;

        let api_url = alloc::format!("http://localhost:3000/license?key={}", license_key);

        let deadline = now.add(Duration::from_millis(5_000));
        let request = http::Request::get(&api_url);
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| "Failed to send license check request")?;
        let response = pending
            .try_wait(deadline)
            .map_err(|_| "License check request deadline reached")?
            .map_err(|_| "License check request failed")?;

        // Update last check timestamp
        storage.set(&now.unix_millis());

        // Check if response is not 200 OR if body doesn't contain valid: true
        let is_valid = if response.code == 200 {
            let body = response.body().collect::<Vec<u8>>();
            let body_str = alloc::str::from_utf8(&body).map_err(|_| "Invalid UTF8 in response")?;

            // Parse JSON response to check if valid: true
            Self::parse_license_response(body_str)
        } else {
            log::error!(
                target: LOG_TARGET,
                "License check failed with HTTP status: {}",
                response.code
            );
            false
        };

        // If license is invalid, request halt
        if !is_valid {
            log::error!(
                target: LOG_TARGET,
                "License validation failed! Requesting block production halt."
            );
            let storage_halt = StorageValueRef::persistent(b"licensed_aura::halt_requested");
            storage_halt.set(&true);
        } else {
            log::info!(
                target: LOG_TARGET,
                "License validation successful."
            );
        }

        Ok(())
    }

    /// Parse the license API response to check if valid: true
    fn parse_license_response(response_str: &str) -> bool {
        // Simple JSON parsing to find "valid":true or "valid": true
        // This is a basic implementation - in production, consider using a proper JSON parser
        if let Some(start) = response_str.find("\"valid\"") {
            let after_valid = &response_str[start + 7..];
            // Skip whitespace and colon
            let trimmed = after_valid.trim_start();
            if let Some(colon_trimmed) = trimmed.strip_prefix(':') {
                let value_part = colon_trimmed.trim_start();
                return value_part.starts_with("true");
            }
        }
        false
    }

    /// Change authorities.
    ///
    /// The storage will be applied immediately.
    /// And aura consensus log will be appended to block's log.
    ///
    /// This is a no-op if `new` is empty.
    pub fn change_authorities(new: BoundedVec<T::AuthorityId, T::MaxAuthorities>) {
        if new.is_empty() {
            log::warn!(target: LOG_TARGET, "Ignoring empty authority change.");

            return;
        }

        <Authorities<T>>::put(&new);

        let log = DigestItem::Consensus(
            AURA_ENGINE_ID,
            ConsensusLog::AuthoritiesChange(new.into_inner()).encode(),
        );
        <frame_system::Pallet<T>>::deposit_log(log);
    }

    /// Initial authorities.
    ///
    /// The storage will be applied immediately.
    ///
    /// The authorities length must be equal or less than T::MaxAuthorities.
    pub fn initialize_authorities(authorities: &[T::AuthorityId]) {
        if !authorities.is_empty() {
            assert!(
                <Authorities<T>>::get().is_empty(),
                "Authorities are already initialized!"
            );
            let bounded = <BoundedSlice<'_, _, T::MaxAuthorities>>::try_from(authorities)
                .expect("Initial authority set must be less than T::MaxAuthorities");
            <Authorities<T>>::put(bounded);
        }
    }

    /// Return current authorities length.
    pub fn authorities_len() -> usize {
        Authorities::<T>::decode_len().unwrap_or(0)
    }

    /// Get the current slot from the pre-runtime digests.
    fn current_slot_from_digests() -> Option<Slot> {
        let digest = frame_system::Pallet::<T>::digest();
        let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());
        for (id, mut data) in pre_runtime_digests {
            if id == AURA_ENGINE_ID {
                return Slot::decode(&mut data).ok();
            }
        }

        None
    }

    /// Determine the Aura slot-duration based on the Timestamp module configuration.
    pub fn slot_duration() -> T::Moment {
        T::SlotDuration::get()
    }

    /// Ensure the correctness of the state of this pallet.
    ///
    /// This should be valid before or after each state transition of this pallet.
    ///
    /// # Invariants
    ///
    /// ## `CurrentSlot`
    ///
    /// If we don't allow for multiple blocks per slot, then the current slot must be less than the
    /// maximal slot number. Otherwise, it can be arbitrary.
    ///
    /// ## `Authorities`
    ///
    /// * The authorities must be non-empty.
    /// * The current authority cannot be disabled.
    /// * The number of authorities must be less than or equal to `T::MaxAuthorities`. This however,
    ///   is guarded by the type system.
    #[cfg(any(test, feature = "try-runtime"))]
    pub fn do_try_state() -> Result<(), sp_runtime::TryRuntimeError> {
        // We don't have any guarantee that we are already after `on_initialize` and thus we have to
        // check the current slot from the digest or take the last known slot.
        let current_slot =
            Self::current_slot_from_digests().unwrap_or_else(|| CurrentSlot::<T>::get());

        // Check that the current slot is less than the maximal slot number, unless we allow for
        // multiple blocks per slot.
        if !T::AllowMultipleBlocksPerSlot::get() {
            frame_support::ensure!(
                current_slot < u64::MAX,
                "Current slot has reached maximum value and cannot be incremented further.",
            );
        }

        let authorities_len =
            <Authorities<T>>::decode_len().ok_or("Failed to decode authorities length")?;

        // Check that the authorities are non-empty.
        frame_support::ensure!(!authorities_len.is_zero(), "Authorities must be non-empty.");

        // Check that the current authority is not disabled.
        let authority_index = *current_slot % authorities_len as u64;
        frame_support::ensure!(
            !T::DisabledValidators::is_disabled(authority_index as u32),
            "Current validator is disabled and should not be attempting to author blocks.",
        );

        Ok(())
    }
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
    type Public = T::AuthorityId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
    type Key = T::AuthorityId;

    fn on_genesis_session<'a, I: 'a>(validators: I)
    where
        I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
    {
        let authorities = validators.map(|(_, k)| k).collect::<Vec<_>>();
        Self::initialize_authorities(&authorities);
    }

    fn on_new_session<'a, I: 'a>(changed: bool, validators: I, _queued_validators: I)
    where
        I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
    {
        // instant changes
        if changed {
            let next_authorities = validators.map(|(_, k)| k).collect::<Vec<_>>();
            let last_authorities = Authorities::<T>::get();
            if last_authorities != next_authorities {
                if next_authorities.len() as u32 > T::MaxAuthorities::get() {
                    log::warn!(
                        target: LOG_TARGET,
                        "next authorities list larger than {}, truncating",
                        T::MaxAuthorities::get(),
                    );
                }
                let bounded = <BoundedVec<_, T::MaxAuthorities>>::truncate_from(next_authorities);
                Self::change_authorities(bounded);
            }
        }
    }

    fn on_disabled(i: u32) {
        let log = DigestItem::Consensus(
            AURA_ENGINE_ID,
            ConsensusLog::<T::AuthorityId>::OnDisabled(i as AuthorityIndex).encode(),
        );

        <frame_system::Pallet<T>>::deposit_log(log);
    }
}

impl<T: Config> FindAuthor<u32> for Pallet<T> {
    fn find_author<'a, I>(digests: I) -> Option<u32>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        for (id, mut data) in digests.into_iter() {
            if id == AURA_ENGINE_ID {
                let slot = Slot::decode(&mut data).ok()?;
                let author_index = *slot % Self::authorities_len() as u64;
                return Some(author_index as u32);
            }
        }

        None
    }
}

/// We can not implement `FindAuthor` twice, because the compiler does not know if
/// `u32 == T::AuthorityId` and thus, prevents us to implement the trait twice.
#[doc(hidden)]
pub struct FindAccountFromAuthorIndex<T, Inner>(core::marker::PhantomData<(T, Inner)>);

impl<T: Config, Inner: FindAuthor<u32>> FindAuthor<T::AuthorityId>
    for FindAccountFromAuthorIndex<T, Inner>
{
    fn find_author<'a, I>(digests: I) -> Option<T::AuthorityId>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        let i = Inner::find_author(digests)?;

        let validators = Authorities::<T>::get();
        validators.get(i as usize).cloned()
    }
}

/// Find the authority ID of the Aura authority who authored the current block.
pub type AuraAuthorId<T> = FindAccountFromAuthorIndex<T, Pallet<T>>;

impl<T: Config> IsMember<T::AuthorityId> for Pallet<T> {
    fn is_member(authority_id: &T::AuthorityId) -> bool {
        Authorities::<T>::get().iter().any(|id| id == authority_id)
    }
}

impl<T: Config> OnTimestampSet<T::Moment> for Pallet<T> {
    fn on_timestamp_set(moment: T::Moment) {
        let slot_duration = Self::slot_duration();
        assert!(
            !slot_duration.is_zero(),
            "Aura slot duration cannot be zero."
        );

        let timestamp_slot = moment / slot_duration;
        let timestamp_slot = Slot::from(timestamp_slot.saturated_into::<u64>());

        assert_eq!(
            CurrentSlot::<T>::get(),
            timestamp_slot,
            "Timestamp slot must match `CurrentSlot`"
        );
    }
}
