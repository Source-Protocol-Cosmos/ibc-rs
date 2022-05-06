use eyre::eyre;
use ibc::events::IbcEvent;
use ibc_proto::google::protobuf::Any;
use ibc_relayer::chain::cosmos::query::account::query_account;
use ibc_relayer::chain::cosmos::tx::estimate_fee_and_send_tx;
use ibc_relayer::chain::cosmos::types::config::TxConfig;
use ibc_relayer::chain::cosmos::types::tx::TxSyncResult;
use ibc_relayer::chain::cosmos::wait::wait_for_block_commits;
use ibc_relayer::config::types::Memo;
use ibc_relayer::keyring::KeyEntry;

use crate::error::Error;

/**
 A simplified version of send_tx that does not depend on `ChainHandle`.

 This allows different wallet ([`KeyEntry`]) to be used for submitting
 transactions. The simple behavior as follows:

 - Query the account information on the fly. This may introduce more
   overhead in production, but does not matter in testing.
 - Do not split the provided messages into smaller batches.
 - Wait for TX sync result, and error if any result contains
   error event.
*/
pub async fn simple_send_tx(
    config: &TxConfig,
    key_entry: &KeyEntry,
    memo: &Memo,
    messages: Vec<Any>,
) -> Result<(), Error> {
    let account = query_account(&config.grpc_address, &key_entry.account)
        .await?
        .into();

    let message_count = messages.len();

    let response = estimate_fee_and_send_tx(
        config,
        key_entry,
        &account,
        &Default::default(),
        memo,
        messages,
    )
    .await?;

    let events_per_tx = vec![IbcEvent::default(); message_count];

    let tx_sync_result = TxSyncResult {
        response,
        events: events_per_tx,
    };

    let mut tx_sync_results = vec![tx_sync_result];

    wait_for_block_commits(
        &config.chain_id,
        &config.rpc_client,
        &config.rpc_address,
        &config.rpc_timeout,
        &mut tx_sync_results,
    )
    .await?;

    for result in tx_sync_results.iter() {
        for event in result.events.iter() {
            if let IbcEvent::ChainError(e) = event {
                return Err(Error::generic(eyre!("send_tx result in error: {}", e)));
            }
        }
    }

    Ok(())
}
