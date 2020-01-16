extern crate couchbase_lite_c;
extern crate uuid;

use serde::{Deserialize, Serialize};
use std::time::Duration;

use couchbase_lite_c::Database;
use couchbase_lite_c::Replicator;

static DATABASE_PATH: &str = "/data/local/tmp";
static TARGET_URL: &str = "ws://127.0.0.1:4984/mydb";

#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prop1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prop2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prop3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prop4: Option<String>,
}

fn replicate(db_name: &str, target_url: &str) {
    println!("\n --- \n");
    let database = Database::open(DATABASE_PATH.to_string(), db_name).unwrap();
    let replicator = Replicator::new(database.clone(), target_url.to_string()).unwrap();
    replicator.start();
    println!("waiting ...");
    std::thread::sleep(Duration::from_secs(5));
    replicator.stop();
    database.close().unwrap();
}

fn display_local_datas(db_name: &str) {
    println!("\n --- \n");
    let database = Database::open(DATABASE_PATH.to_string(), db_name).unwrap();
    //let query = database.new_query("SELECT _id, id, prop1 AS person WHERE first_name='Scott'".to_string());
    let query = database.new_query("SELECT _id AS id, _rev as rev, * AS person".to_string()).unwrap();
    println!("================================");
    let rs = query.execute().unwrap();
    while rs.has_next() {
        println!(
            "===> _id: {:?} -  _rev: {:?} - person: {:?}",
            rs.value("id".to_string()),
            rs.value("rev".to_string()),
            rs.value("person".to_string())
        );
    }
    database.close().unwrap();
}

fn populate_local_database(db_name: &str) {
    println!("\n --- \n");
    let database = Database::open(DATABASE_PATH.to_string(), db_name).unwrap();
    let doc_id = String::from("foo2");
    database.in_batch(&|| {
        match database.get_document(doc_id.clone()) {
            Some(doc) => println!("Doc already exits: {:?}", doc.jsonify()),
            None => {
                database
                    .in_batch(&|| {
                        let document = database.create_document(doc_id.clone());
                        println!("Document ID: {:?}", document.id());
                        let person = Person {
                            first_name: "Scott".to_string(),
                            last_name: "Tiger".to_string(),
                            prop1: None,
                            prop2: None,
                            prop3: None,
                            prop4: None,
                        };
                        document.fill(serde_json::to_string_pretty(&person).unwrap()).unwrap();

                        //document.set_value("first_name".to_string(), "Scott".to_string());
                        //document.set_value("last_name".to_string(), "Tiger".to_string());
                        database.save_document(document).unwrap();
                    })
                    .unwrap();
            }
        };
    }).unwrap();
    database.close().unwrap();
}

fn update_local_database(db_name: &str) {
    println!("\n --- \n");
    let database = Database::open(DATABASE_PATH.to_string(), db_name).unwrap();
    let doc_id = String::from("foo");
    database.in_batch(&|| {
        if let Some(doc) = database.get_document(doc_id.clone()) {
            println!("Doc already exits: {:?}", doc.jsonify());
            let json = doc.jsonify();
            let mut data: Person = serde_json::from_str(json.as_str()).unwrap();
            if data.prop1.is_none() {
                data.prop1 = Some(format!("{}_val1", db_name));
            } else if data.prop2.is_none() {
                data.prop2 = Some(format!("{}_val2", db_name));
            } else if data.prop3.is_none() {
                data.prop3 = Some(format!("{}_val3", db_name));
            } else if data.prop4.is_none() {
                data.prop4 = Some(format!("{}_val4", db_name));
            }
            //let uuid = Uuid::new_v4().to_string();
            //let property_name = format!("new_property_{}", uuid);
            //doc.set_value("Bob".to_string(), property_name);
            doc.fill(serde_json::to_string_pretty(&data).unwrap()).unwrap();
            println!("Modify existing doc: {:?}", doc.jsonify());
            database.save_document(doc).unwrap();
        }
    }).unwrap();
    database.close().unwrap();
}

fn main() {
    if let Err(cause) = std::panic::catch_unwind(|| {
        // start replication thread
        let db_name = "mydba";
        replicate(db_name, TARGET_URL);
        display_local_datas(db_name);
        populate_local_database(db_name);
        display_local_datas(db_name);
        update_local_database(db_name);
        display_local_datas(db_name);
        replicate(db_name, TARGET_URL);
        display_local_datas(db_name);
    }) {
        println!("Code suffered a panic, cause = {:?}", cause);
    }
}

/*
match database.get_document(doc_id.clone()) {
    Some(doc) => {
        println!("Document ID: {:?}", doc.id());
        let mut person: Person = serde_json::from_str(doc.jsonify().as_str()).unwrap();
        person.last_name = "Bob".to_string();
        doc.fill(serde_json::to_string_pretty(&person).unwrap());
        database.save_document(doc);

        let final_doc = database.get_document(doc_id.clone()).unwrap();
        println!("-- RESULT -- Document ID: {:?} - {:?}", final_doc.id(), final_doc.jsonify());

        println!("================================");
        //let query = database.new_query("SELECT _id, id, prop1 AS person WHERE first_name='Scott'".to_string());
        let query = database.new_query("SELECT _id AS id, * AS person".to_string()).unwrap();
        println!("================================");
        let rs = query.execute().unwrap();
        while rs.has_next() {
            println!(
                "===> _id: {:?} - person: {:?}",
                rs.value("id".to_string()),
                rs.value("person".to_string())
            );
        }
        println!("================================ ... ");
        //database.close().unwrap();
        let db_name = "mydb1";
        let database = Database::open(DATABASE_PATH.to_string(), db_name).unwrap();

        let replicator = Replicator::new(database).unwrap();
        println!("================================ Start replicator");
        replicator.start();
        println!("================================");
        println!("waiting ...");
        std::thread::sleep(Duration::from_secs(60));
        println!("================================");
        //database.close().unwrap();
    }
    None => {}
};
*/
