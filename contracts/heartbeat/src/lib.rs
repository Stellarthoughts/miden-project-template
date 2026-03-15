#![no_std]
#![feature(alloc_error_handler)]

use miden::{component, felt, tx, Felt, Value, ValueAccess};

/// Heartbeat component - creator pings periodically to prove they're alive.
/// Used alongside time-capsule notes for dead-man-switch functionality.
#[component]
struct Heartbeat {
    /// Block height of the last ping
    #[storage(description = "block height of last heartbeat ping")]
    last_checkin: Value,

    /// Maximum blocks between pings before considered dead
    #[storage(description = "heartbeat interval in blocks")]
    interval: Value,
}

#[component]
impl Heartbeat {
    /// Record a heartbeat at the current block height.
    pub fn ping(&mut self) -> Felt {
        let current_block = tx::get_block_number();
        self.last_checkin.write(current_block);
        current_block
    }

    /// Get the block height of the last heartbeat ping.
    pub fn get_last_checkin(&self) -> Felt {
        self.last_checkin.read()
    }

    /// Get the heartbeat interval.
    pub fn get_interval(&self) -> Felt {
        self.interval.read()
    }

    /// Set the heartbeat interval (in blocks).
    /// Returns the new interval.
    pub fn set_interval(&mut self, new_interval: Felt) -> Felt {
        self.interval.write(new_interval);
        new_interval
    }

    /// Check if the creator is considered alive.
    /// Returns 1 if alive (current_block - last_checkin < interval), 0 if dead.
    pub fn is_alive(&self) -> Felt {
        let current_block = tx::get_block_number();
        let last: Felt = self.last_checkin.read();
        let interval: Felt = self.interval.read();

        if current_block.as_u64() - last.as_u64() < interval.as_u64() {
            felt!(1)
        } else {
            felt!(0)
        }
    }
}
