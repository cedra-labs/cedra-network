// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use cedra_infallible::RwLock;
use cedra_logger::{cedra_logger::CedraData, info, Writer};
use std::sync::Arc;

#[derive(Default)]
struct VecWriter {
    logs: Arc<RwLock<Vec<String>>>,
}

impl Writer for VecWriter {
    fn write(&self, log: String) {
        self.logs.write().push(log)
    }

    fn write_buferred(&mut self, log: String) {
        self.write(log);
    }
}

#[test]
fn verify_end_to_end() {
    let writer = VecWriter::default();
    let logs = writer.logs.clone();
    CedraData::builder()
        .is_async(false)
        .printer(Box::new(writer))
        .build();

    assert_eq!(logs.read().len(), 0);
    info!("Hello");
    assert_eq!(logs.read().len(), 1);
    let string = logs.write().remove(0);
    assert!(string.contains("INFO"));
    assert!(string.ends_with("Hello"));
    info!(foo = 5, bar = 10, foobar = 15);
    let string = logs.write().remove(0);
    let expect = r#"{"bar":10,"foo":5,"foobar":15}"#;
    assert!(string.ends_with(expect));
}
