use integration::helpers::{
    build_project_in_dir, create_basic_wallet_account, create_note_from_package,
    setup_local_client, AccountCreationConfig, ClientSetup, NoteCreationConfig,
};

use anyhow::{Context, Result};
use miden_client::{
    transaction::{OutputNote, TransactionRequestBuilder},
    Felt,
};
use std::{path::Path, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Chronovault Local Node Validation ===\n");

    // 1. Setup local client
    let ClientSetup {
        mut client,
        keystore,
    } = setup_local_client().await?;

    let sync_summary = client.sync_state().await?;
    println!(
        "[✓] Connected to local node. Latest block: {}",
        sync_summary.block_num
    );

    // 2. Build the time-capsule-note contract
    println!("\n--- Building contracts ---");
    let note_package = Arc::new(
        build_project_in_dir(Path::new("../contracts/time-capsule-note"), true)
            .context("Failed to build time-capsule-note contract")?,
    );
    println!("[✓] time-capsule-note built");

    // 3. Create sender wallet (the capsule creator)
    let sender_cfg = AccountCreationConfig::default();
    let sender = create_basic_wallet_account(&mut client, keystore.clone(), sender_cfg)
        .await
        .context("Failed to create sender wallet")?;
    println!("[✓] Sender account created: {}", sender.id().to_hex());

    // 4. Create recipient wallet
    let recipient_cfg = AccountCreationConfig::default();
    let recipient = create_basic_wallet_account(&mut client, keystore.clone(), recipient_cfg)
        .await
        .context("Failed to create recipient wallet")?;
    println!("[✓] Recipient account created: {}", recipient.id().to_hex());

    // 5. Create a time capsule note
    // Input layout: [0]=unlock_timestamp, [1]=unlock_height,
    //               [2]=recipient_prefix, [3]=recipient_suffix, [4..]=message
    println!("\n--- Creating time capsule ---");
    let message: Vec<Felt> = "Hello from the past!"
        .bytes()
        .map(|b| Felt::new(b as u64))
        .collect();

    let mut inputs = vec![
        Felt::new(0),                      // unlock_timestamp (disabled — using block height)
        Felt::new(0),                      // unlock_height (already past for demo)
        recipient.id().prefix().as_felt(), // recipient prefix
        recipient.id().suffix(),           // recipient suffix
    ];
    inputs.extend(message);

    let capsule_note = create_note_from_package(
        &mut client,
        note_package.clone(),
        sender.id(),
        NoteCreationConfig {
            inputs,
            ..Default::default()
        },
    )
    .context("Failed to create time capsule note")?;
    println!("[✓] Time capsule note created: {}", capsule_note.id().to_hex());

    // 6. Publish the note (sender submits transaction to put note on-chain)
    println!("\n--- Publishing capsule to network ---");
    let publish_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(capsule_note.clone())])
        .build()
        .context("Failed to build publish transaction request")?;

    let publish_tx_id = client
        .submit_new_transaction(sender.id(), publish_request)
        .await
        .context("Failed to submit publish transaction")?;
    println!("[✓] Publish transaction submitted: {}", publish_tx_id.to_hex());

    // 7. Sync state to pick up the published note
    client
        .sync_state()
        .await
        .context("Failed to sync state after publishing")?;
    println!("[✓] State synced after publish");

    // 8. Consume the capsule (recipient opens it)
    println!("\n--- Recipient opening capsule ---");
    let consume_request = TransactionRequestBuilder::new()
        .input_notes([(capsule_note.clone(), None)])
        .build()
        .context("Failed to build consume transaction request")?;

    let consume_tx_id = client
        .submit_new_transaction(recipient.id(), consume_request)
        .await
        .context("Failed to submit consume transaction")?;
    println!("[✓] Consume transaction submitted: {}", consume_tx_id.to_hex());

    // 9. Final sync
    client
        .sync_state()
        .await
        .context("Failed to final sync")?;
    println!("[✓] Final state synced");

    println!("\n=== ALL VALIDATIONS PASSED ===");
    println!(
        "Time capsule created, published, and successfully consumed by recipient."
    );

    Ok(())
}
