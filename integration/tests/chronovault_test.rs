use integration::helpers::{
    build_project_in_dir, create_testing_note_from_package, NoteCreationConfig,
};

use miden_client::{account::AccountId, transaction::OutputNote, Felt};
use miden_testing::{Auth, MockChain};
use std::{path::Path, sync::Arc};

/// Max safe Felt value — used as "never" sentinel to disable unlock conditions.
/// Goldilocks prime field: p = 2^64 - 2^32 + 1, max value = p - 1.
const NEVER: u64 = 18_446_744_069_414_584_320;

/// Build capsule inputs with layout:
/// [0] unlock_timestamp, [1] unlock_height,
/// [2] recipient prefix, [3] recipient suffix, [4..] message
fn capsule_inputs(
    unlock_timestamp: u64,
    unlock_height: u64,
    recipient: &AccountId,
) -> Vec<Felt> {
    vec![
        Felt::new(unlock_timestamp),
        Felt::new(unlock_height),
        recipient.prefix().as_felt(),
        recipient.suffix(),
        // "Hello" as ASCII felts
        Felt::new(72),
        Felt::new(101),
        Felt::new(108),
        Felt::new(108),
        Felt::new(111),
    ]
}

#[tokio::test]
async fn test_block_height_unlock() -> anyhow::Result<()> {
    let mut builder = MockChain::builder();

    let sender = builder.add_existing_wallet(Auth::BasicAuth)?;
    let recipient = builder.add_existing_wallet(Auth::BasicAuth)?;

    let note_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/time-capsule-note"),
        true,
    )?);

    // unlock_timestamp=NEVER (disabled), unlock_height=0 (already past)
    let capsule_note = create_testing_note_from_package(
        note_package,
        sender.id(),
        NoteCreationConfig {
            inputs: capsule_inputs(NEVER, 0, &recipient.id()),
            ..Default::default()
        },
    )?;

    builder.add_output_note(OutputNote::Full(capsule_note.clone()));

    let mut mock_chain = builder.build()?;
    let tx_context = mock_chain
        .build_tx_context(recipient.id(), &[capsule_note.id()], &[])?
        .build()?;

    let executed_tx = tx_context.execute().await?;
    mock_chain.add_pending_executed_transaction(&executed_tx)?;
    mock_chain.prove_next_block()?;

    Ok(())
}

#[tokio::test]
async fn test_capsule_sealed_too_early() -> anyhow::Result<()> {
    let mut builder = MockChain::builder();

    let sender = builder.add_existing_wallet(Auth::BasicAuth)?;
    let recipient = builder.add_existing_wallet(Auth::BasicAuth)?;

    let note_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/time-capsule-note"),
        true,
    )?);

    // Both conditions set to NEVER — should NOT unlock
    let capsule_note = create_testing_note_from_package(
        note_package,
        sender.id(),
        NoteCreationConfig {
            inputs: capsule_inputs(NEVER, NEVER, &recipient.id()),
            ..Default::default()
        },
    )?;

    builder.add_output_note(OutputNote::Full(capsule_note.clone()));

    let mock_chain = builder.build()?;
    let tx_context = mock_chain
        .build_tx_context(recipient.id(), &[capsule_note.id()], &[])?
        .build()?;

    let execute_result = tx_context.execute().await;
    let err = execute_result.expect_err(
        "Capsule consumption should fail when all unlock conditions are in the future",
    );
    println!("Expected failure for early open attempt: {err:#}");

    Ok(())
}

#[tokio::test]
async fn test_wrong_recipient_rejected() -> anyhow::Result<()> {
    let mut builder = MockChain::builder();

    let sender = builder.add_existing_wallet(Auth::BasicAuth)?;
    let intended_recipient = builder.add_existing_wallet(Auth::BasicAuth)?;
    let imposter = builder.add_existing_wallet(Auth::BasicAuth)?;

    let note_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/time-capsule-note"),
        true,
    )?);

    // Unlockable (height=0) but wrong recipient
    let capsule_note = create_testing_note_from_package(
        note_package,
        sender.id(),
        NoteCreationConfig {
            inputs: capsule_inputs(NEVER, 0, &intended_recipient.id()),
            ..Default::default()
        },
    )?;

    builder.add_output_note(OutputNote::Full(capsule_note.clone()));

    let mock_chain = builder.build()?;
    let tx_context = mock_chain
        .build_tx_context(imposter.id(), &[capsule_note.id()], &[])?
        .build()?;

    let execute_result = tx_context.execute().await;
    let err =
        execute_result.expect_err("Capsule consumption should fail for a non-designated recipient");
    println!("Expected failure for wrong recipient: {err:#}");

    Ok(())
}

#[tokio::test]
async fn test_all_zero_immediately_openable() -> anyhow::Result<()> {
    let mut builder = MockChain::builder();

    let sender = builder.add_existing_wallet(Auth::BasicAuth)?;
    let recipient = builder.add_existing_wallet(Auth::BasicAuth)?;

    let note_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/time-capsule-note"),
        true,
    )?);

    // Both conditions set to 0 — capsule should be immediately openable
    let capsule_note = create_testing_note_from_package(
        note_package,
        sender.id(),
        NoteCreationConfig {
            inputs: capsule_inputs(0, 0, &recipient.id()),
            ..Default::default()
        },
    )?;

    builder.add_output_note(OutputNote::Full(capsule_note.clone()));

    let mut mock_chain = builder.build()?;
    let tx_context = mock_chain
        .build_tx_context(recipient.id(), &[capsule_note.id()], &[])?
        .build()?;

    let executed_tx = tx_context.execute().await?;
    mock_chain.add_pending_executed_transaction(&executed_tx)?;
    mock_chain.prove_next_block()?;

    Ok(())
}
