use api::PublicApi;
use exonum::{
    api::ServiceApiBuilder,
    helpers::fabric::{ServiceFactory, Context},
    crypto::Hash,
    storage::Snapshot,
    blockchain::{Transaction, TransactionSet, Service, ServiceContext},
    messages::RawTransaction,
    encoding,
};
use blockchain::{
    transactions::{ExonumRngTransactions, TxPublishVdfResult, TxPublishSeedCommitment},
    schema::RngSchema,
};

use rng::calculate_vdf;
use rand::{self, Rng};

pub const SERVICE_ID: u16 = 9000;
pub const SERVICE_NAME: &str = "exonum_rng";

#[derive(Debug, Default)]
pub struct ExonumRngService;

impl ExonumRngService {
    pub fn new() -> ExonumRngService {
        ExonumRngService
    }
}

impl Service for ExonumRngService {
    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn service_name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn state_hash(&self, view: &Snapshot) -> Vec<Hash> {
        let schema = RngSchema::new(view);
        schema.state_hash()
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        let tx = ExonumRngTransactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    /// Creates transaction after commit of the block.
    fn after_commit(&self, context: &ServiceContext) {
        // The transaction must be created by the validator.
        if context.validator_id().is_none() {
            return;
        }

        // If there were seed commitment from current validator,
        // and there's a combined seed that is combined from commitments of majority of validators,
        // then calculate our value of VDF and publish it.
        //
        // Otherwise, do nothing.

        let schema = RngSchema::new(context.snapshot());

        // Do nothing if there wasn't a commitment from current validator
        if !schema.validators_seed_commitments().contains(&context.public_key()) {
            // Send validator's seed commitment
            let mut rng = rand::thread_rng();
            let sc = rng.gen::<u64>().to_string();

            //println!("Validator is sending seed commitment {}", sc);
            let (pub_key, sec_key) = (*context.public_key(), context.secret_key().clone());
            context
                .transaction_sender()
                .send(Box::new(TxPublishSeedCommitment::new(
                    &pub_key,
                    &sc,
                    &sec_key,
                )))
                .unwrap();

            return
        }

        if let Some(seed) = schema.last_seed().get() {
            println!("Calculating VDF from seed...");

            if let Some(value) = calculate_vdf(&seed) {
                //println!("VDF calculated by {:?} = {}", *context.public_key(), value);

                let (pub_key, sec_key) = (*context.public_key(), context.secret_key().clone());
                context
                    .transaction_sender()
                    .send(Box::new(TxPublishVdfResult::new(
                        &pub_key,
                        &seed,
                        &value,
                        &sec_key,
                    )))
                    .unwrap();
            } else {
                println!("error: invalid seed value, can't be parsed as hex value in `rug::Integer`: {}", seed);
            }
        }
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        PublicApi::wire(builder);
    }
}

impl ServiceFactory for ExonumRngService {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn make_service(&mut self, _: &Context) -> Box<Service> {
        Box::new(ExonumRngService::new())
    }
}