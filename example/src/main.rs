extern crate couchbase_lite_c;

use std::time::Duration;

use serde::{Deserialize, Serialize};

use couchbase_lite_c::Database;
use couchbase_lite_c::Document;
use couchbase_lite_c::Replicator;


#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let database = Database::open(String::from("/tmp"), "mydb").unwrap();
    let doc_id = String::from("foo");
    let document = match database.get_document(doc_id.clone()){
        Some(doc) => doc,
        None => database.create_document(doc_id.clone())
    };
    println!("Document ID: {:?}", document.id());

    let person = Person {
        first_name: "Scott".to_string(),
        last_name: "Tiger".to_string(),
    };
    document.fill(serde_json::to_string_pretty(&person).unwrap());
    database.save_document(document).unwrap();

    match database.get_document(doc_id.clone()){
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
            let rs = query.execute();
            while rs.has_next() {
                println!("===> _id: {:?} - person: {:?}", rs.value("id".to_string()), rs.value("person".to_string()));
            }
            println!("================================");
            let replicator = Replicator::new(database);
            println!("================================");
            replicator.start();
            println!("================================");
            println!("waiting ...");
            std::thread::sleep(Duration::from_secs(15));
            println!("================================");
        },
        None => {}
    };


}
