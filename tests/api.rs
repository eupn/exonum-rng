extern crate exonum;
extern crate exonum_rng;
extern crate exonum_testkit;
extern crate exonum_configuration;
//#[macro_use]
//extern crate serde_json;

/*
struct ExonumRngApi {
    pub inner: TestKitApi,
}

impl ExonumRngApi {
    fn assert_tx_status(&self, tx_hash: Hash, expected_status: &serde_json::Value) {
        let info: serde_json::Value = self.inner
            .public(ApiKind::Explorer)
            .query(&TransactionQuery::new(tx_hash))
            .get("v1/transactions")
            .unwrap();

        if let serde_json::Value::Object(mut info) = info {
            let tx_status = info.remove("status").unwrap();
            assert_eq!(tx_status, *expected_status);
        } else {
            panic!("Invalid transaction info format, object expected");
        }
    }

    fn assert_tx_success(&self, tx_hash: Hash) {
        self.assert_tx_status(tx_hash, &json!({ "type": "success" }));
    }
}

fn create_testkit() -> (TestKit, ExonumRngApi) {
    let testkit = TestKitBuilder::validator()
        .with_service(ExonumRngService)
        .create();

    let api = ExonumRngApi {
        inner: testkit.api(),
    };
    (testkit, api)
}
*/
