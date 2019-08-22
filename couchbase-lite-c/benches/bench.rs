#[macro_use]
extern crate criterion;

use couchbase_lite_c::{Database, Document};
use criterion::Criterion;
use serde::{Deserialize, Serialize};
use std::fs;

fn test_dir() -> String {
    let dir = "/tmp/benchdb".to_string();
    match fs::create_dir_all(dir.clone()) {
        Ok(_) => {}
        Err(e) => panic!("Cannot create database directory: {:?}", e),
    };
    dir
}

fn open_database() -> Database {
    let database_name = String::from("testdb");
    Database::open(test_dir(), &database_name).unwrap()
}

fn save_document_from_json() {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }
    let person = Person {
        first_name: "James".to_string(),
        last_name: "Bomb".to_string(),
    };

    let database = open_database();
    let doc_id = String::from("foo");
    let doc = Document::new(doc_id.clone());
    doc.fill(serde_json::to_string_pretty(&person).unwrap());
    assert_eq!("{\"first_name\":\"James\",\"last_name\":\"Bomb\"}", doc.jsonify());

    let saved = database.save_document(doc);
    //assert_eq!(true, saved.is_ok());
    //let saved = saved.unwrap();
    //assert_eq!(doc_id, saved.id());
    //assert_eq!(1, saved.sequence());
    //assert_eq!("{\"first_name\":\"James\",\"last_name\":\"Bomb\"}", saved.jsonify());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("save document from json", |b| b.iter(|| save_document_from_json()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
