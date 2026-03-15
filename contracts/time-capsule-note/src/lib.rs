#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use miden::*;

/// Time capsule note - a sealed digital capsule that holds assets and a message.
///
/// Dual unlock conditions (either satisfies):
/// - Timestamp unlock: opens when block_timestamp >= unlock_timestamp
/// - Block height unlock: opens when current_block >= unlock_height
///
/// To disable a condition, set it to the max Felt value (18446744069414584320).
/// A value of 0 means "already past" (condition is immediately satisfied).
///
/// Note inputs layout:
/// [0] - unlock_timestamp (Felt): unix seconds for time-based unlock (max = disabled)
/// [1] - unlock_height (Felt): absolute block height for block-based unlock (max = disabled)
/// [2] - recipient account ID prefix (Felt)
/// [3] - recipient account ID suffix (Felt)
/// [4..] - message payload as Felt-encoded values (optional)
#[note]
struct TimeCapsuleNote;

#[note]
impl TimeCapsuleNote {
    #[note_script]
    fn run(self, _arg: Word) {
        let inputs = active_note::get_inputs();

        // Parse unlock conditions
        let unlock_timestamp = inputs[0];
        let unlock_height = inputs[1];

        // Parse recipient account ID (two felts)
        let recipient_prefix = inputs[2];
        let recipient_suffix = inputs[3];

        let current_block = tx::get_block_number();
        let current_timestamp = tx::get_block_timestamp();

        // --- Unlock condition check ---
        // Either condition being met is sufficient.
        // To disable a condition, set it to the max Felt value (effectively never triggers).
        // A value of 0 means "already unlockable" (current >= 0 is always true).
        let timestamp_unlocked = current_timestamp.as_u64() >= unlock_timestamp.as_u64();
        let height_unlocked = current_block.as_u64() >= unlock_height.as_u64();

        assert!(
            timestamp_unlocked || height_unlocked,
            "Capsule is still sealed: neither time lock nor block height condition satisfied"
        );

        // --- Recipient check ---
        // Verify the consuming account is the designated recipient
        let consumer_id = active_account::get_id();
        let expected_id = AccountId::new(recipient_prefix, recipient_suffix);
        assert_eq!(
            consumer_id, expected_id,
            "Only the designated recipient can open this capsule"
        );

        // --- Transfer all assets to recipient ---
        for asset in active_note::get_assets() {
            native_account::add_asset(asset);
        }

        // Message payload (inputs[4..]) is readable by the recipient's client
        // from the note inputs after consumption - no on-chain action needed.
    }
}
