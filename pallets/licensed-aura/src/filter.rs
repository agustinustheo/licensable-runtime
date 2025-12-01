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

//! BaseCallFilter implementation for Licensed Aura
//!
//! This filter enforces "empty blocks while halted" - when the Licensed Aura
//! pallet is in a halted state, only specific whitelisted calls are allowed.

use super::*;
use frame_support::traits::Contains;
use log::{error, warn};

const LOG_TARGET: &str = "licensed-aura";

/// Filter that enforces "empty blocks while halted".
///
/// When the Licensed Aura pallet is halted (license invalid or manually halted),
/// this filter blocks all extrinsics except:
/// - Mandatory inherents (like timestamp)
/// - Resume production calls (sudo_resume_production)
/// - Halt production calls (offchain_worker_halt_production)
pub struct AuraHaltFilter<RuntimeCall, T>(core::marker::PhantomData<(RuntimeCall, T)>);

impl<RuntimeCall, T> AuraHaltFilter<RuntimeCall, T>
where
    T: Config,
    RuntimeCall: IsLicensedAuraCall + IsTimestampCall + IsSudoCall<RuntimeCall>,
{
    /// Helper: what is allowed *while halted*?
    fn allowed_while_halted(call: &RuntimeCall) -> bool {
        match () {
            // Direct calls to the licensed aura pallet.
            _ if call.is_sudo_resume_production() => true,
            _ if call.is_offchain_worker_halt() => true,
            _ if call.is_offchain_worker_resume() => true,

            // Sudo wrapping an allowed call: sudo(Aura::sudo_resume_production { .. })
            _ if call.is_sudo_wrapping_allowed() => true,

            // Everything else is NOT allowed while halted.
            _ => false,
        }
    }
}

impl<RuntimeCall, T> Contains<RuntimeCall> for AuraHaltFilter<RuntimeCall, T>
where
    T: Config,
    RuntimeCall: IsLicensedAuraCall + IsTimestampCall + IsSudoCall<RuntimeCall> + core::fmt::Debug,
{
    fn contains(call: &RuntimeCall) -> bool {
        // Always allow mandatory inherents (like timestamp).
        // This keeps block production working even while halted.
        if call.is_timestamp_set() {
            return true;
        }

        // Everything else is governed by the halt flag.
        let halted = Pallet::<T>::is_halted();

        if halted {
            // Only log when we're actually *blocking* something, not for allowed ones.
            if !Self::allowed_while_halted(call) {
                warn!(
                    target: LOG_TARGET,
                    "❗️ Licensed Aura is halted. Please renew your license."
                );
                error!(
                    target: LOG_TARGET,
                    "❌️ Licensed Aura is halted. Extrinsic {:?} cannot be processed.",
                    call
                );
            }

            // Only allow the whitelisted calls while halted.
            Self::allowed_while_halted(call)
        } else {
            // Normal mode: allow everything.
            true
        }
    }
}

/// Trait to check if a RuntimeCall is a call to the licensed aura pallet
pub trait IsLicensedAuraCall {
    /// Check if this is a sudo_resume_production call
    fn is_sudo_resume_production(&self) -> bool;
    /// Check if this is an offchain_worker_halt_production call
    fn is_offchain_worker_halt(&self) -> bool;
    /// Check if this is an offchain_worker_resume_production call
    fn is_offchain_worker_resume(&self) -> bool;
}

/// Trait to check if a RuntimeCall is a timestamp::set call
pub trait IsTimestampCall {
    /// Check if this is a timestamp::set call
    fn is_timestamp_set(&self) -> bool;
}

/// Trait to check if a RuntimeCall is a sudo call wrapping another call
pub trait IsSudoCall<RuntimeCall> {
    /// Check if this is a sudo call wrapping an allowed call (resume or halt)
    fn is_sudo_wrapping_allowed(&self) -> bool;
}
