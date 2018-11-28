use exonum::{
    crypto::Hash,
    blockchain::Transaction,
    node::TransactionSend,
    api::{ServiceApiState, Result as ApiResult, ServiceApiBuilder},
};
use blockchain::transactions::ExonumRngTransactions;

#[derive(Debug, Serialize)]
struct TxResult {
    tx_hash: Hash
}

fn post_transaction(state: &ServiceApiState, tx: ExonumRngTransactions) -> ApiResult<TxResult> {
    let transaction: Box<dyn Transaction> = tx.into();
    let tx_hash = transaction.hash();
    state.sender().send(transaction)?;
    Ok(TxResult { tx_hash })
}

#[derive(Clone)]
pub struct PublicApi;

impl PublicApi {
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint_mut("/tx", post_transaction);
    }
}