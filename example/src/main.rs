//extern crate libc;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use core::ptr;
use std::mem;
use std::os::raw::c_void;
use std::str;

use crate::model::Person;

use std::time::Duration;

extern "C" {
    pub fn CBL_Release(document: *const CBLDocument);
}

pub use bindings::*;
use std::slice;

mod bindings;

mod model;

/// Convert a native string to a Rust string
fn to_string(pointer: *const c_char) -> String {
    let slice = unsafe { CStr::from_ptr(pointer).to_bytes() };
    str::from_utf8(slice).unwrap().to_string()
}

/// Convert a Rust string to a native string
fn to_ptr(string: String) -> *const c_char {
    let cs = CString::new(string.as_bytes()).unwrap();
    let ptr = cs.as_ptr();
    // Tell Rust not to clean up the string while we still have a pointer to it.
    // Otherwise, we'll get a segfault.
    mem::forget(cs);
    ptr
}


struct Document/*<T>*/{
    doc: *mut CBLDocument,
    db: *mut CBLDatabase,
    //properties: T
}

impl Document{
    pub fn from_raw(db: *mut CBLDatabase, doc: *mut CBLDocument) -> Self{
        println!("Document::from_raw {:?}", doc);
        Document{
            db: db,
            doc: doc,
        }
    }

    pub fn id(&self) -> String{
        println!("====> 1. {:?} ", self.doc);
        let doc_id = unsafe { CBLDocument_ID(self.doc) };
        println!("====> {:?}", doc_id);
        to_string(doc_id)
    }

    // TODO add generic T: Serialize
    // putProperties
    fn fill(&self, json: String) -> bool {
        let mut error: CBLError = unsafe { std::mem::uninitialized() };
        let json_string = to_ptr(json);
        let status = unsafe { CBLDocument_SetPropertiesAsJSON(self.doc, json_string, &mut error) };
        println!("jsonify {:?} - error: {:?}", status, error);
        status
    }

    // getProperties
    fn jsonify(&self) -> String {
        let json: *mut ::std::os::raw::c_char = unsafe { CBLDocument_PropertiesAsJSON(self.doc) };
        to_string(json)
    }

    //TODO implement Deref and call unsafe { CBL_Release(saved) };

    fn set_value(&self, value: String, for_key: String) {
        unsafe {
            let properties = CBLDocument_MutableProperties(self.doc);

            let key_str = to_ptr(for_key.clone());
            let key = FLString {
                buf: key_str as *const c_void,
                //size: libc::strlen(key_str),
                size: for_key.len()
            };
//            println!("//////////////////// {:?} - {:?}", for_key.len(), libc::strlen(key_str));

            let fl_value = FLDict_Get(properties, key);
            let fl_string = FLValue_AsString(fl_value);
            println!("//////////////////// {:?} - {:?}", properties, key);
            let fl_slot = FLMutableDict_Set(properties, key);

            let val = FLString {
                buf: to_ptr(value.clone()) as *const c_void,
                size: value.len()+1
            };
            FLSlot_SetString(fl_slot, val);

        }

    }

    // TODO add patch or update method and use Fleece JSON Delta API
}

struct Database{
    db : *mut CBLDatabase,
}

impl Database{
    pub fn open(name: &str) -> Self{
        let mut error: CBLError = unsafe { std::mem::uninitialized() };
        let database_name = to_ptr(name.to_string());
        let kCBLDatabase_Create: CBLDatabaseFlags = 1;
        // No encryption (default)
        let kCBLEncryptionNone: CBLEncryptionAlgorithm = 0;
        let config = CBLDatabaseConfiguration {
            directory: to_ptr(String::from("/tmp")),
            flags: kCBLDatabase_Create,
            encryptionKey: CBLEncryptionKey {
                algorithm: kCBLEncryptionNone,
                bytes: [0; 32usize],
            },
        };
        //println!("config {:?}", config);
        let db = unsafe { CBLDatabase_Open(database_name, &config, &mut error) };
        println!("database {:?} - error: {:?}", db, error);
        Database{
            db: db
        }
    }

    // call documentWithID -> Create or Get Existing
    pub fn create_document(&self, id: &str) -> Document {
        let doc_id = to_ptr(id.to_string());
        let doc = unsafe { CBLDocument_New(doc_id) };
        println!("document {:?}", doc);
        Document::from_raw(self.db, doc)
    }

    pub fn get_document(&self, id: &str) -> Option<Document> {
        // TODO return Option
        let doc_id = to_ptr(id.to_string());
        let mut doc = unsafe { CBLDatabase_GetMutableDocument(self.db, doc_id) };
        if doc.is_null() {
            None
        }else {
            Some(Document::from_raw(self.db, doc))
        }
    }

    pub fn save_document(&self, document: Document) {
        let mut error: CBLError = unsafe { std::mem::uninitialized() };
        let kCBLConcurrencyControlFailOnConflict: CBLConcurrencyControl = 0;
        let saved: *const CBLDocument = unsafe { CBLDatabase_SaveDocument(self.db, document.doc, kCBLConcurrencyControlFailOnConflict, &mut error) };
        println!("saved {:?} - error: {:?}", saved, error);
        println!("#### Count {:?}", unsafe { CBLDatabase_Count(self.db) });
        if saved != ptr::null() {
            let json: *mut ::std::os::raw::c_char = unsafe { CBLDocument_PropertiesAsJSON(saved) };
            println!("====> SAVED doc as json {:?}", to_string(json));
        } else {
            println!("ERROR : Cannot saved");
        }
    }

    /*
    Replication push = database.createPushReplication(url);
    Replication pull = database.createPullReplication(url);
    push.setContinuous(true);
    pull.setContinuous(true);

    // Start replicators
    push.start();
    pull.start();
    */

    pub fn new_query(&self, n1ql_query: String) -> Query{
        //kCBLN1QLLanguage
        let n1ql_query_language : CBLQueryLanguage = 1;
        let query_string = to_ptr(n1ql_query);
        let mut outErrorPos : ::std::os::raw::c_int = 0;
        let mut error: CBLError = unsafe { std::mem::uninitialized() };
        let query = unsafe{ CBLQuery_New(self.db, n1ql_query_language, query_string, &mut outErrorPos, &mut error) };
        println!("query {:?} - outErrorPos {:?} - error: {:?}", query, outErrorPos, error);
        Query{
            query: query,
        }
    }
}

struct Query{
    query : *mut CBLQuery,
}

impl Query{

    pub fn execute(&self) -> ResultSet {
        let mut error: CBLError = unsafe { std::mem::uninitialized() };
        let rs = unsafe{ CBLQuery_Execute(self.query, &mut error) };
        println!("rs {:?} - error: {:?}", rs, error);
        ResultSet{
            rs: rs,
        }
    }

}

struct ResultSet{
    rs: *mut CBLResultSet,
}

impl ResultSet{

    pub fn has_next(&self) -> bool{
        let next = unsafe{ CBLResultSet_Next(self.rs) };
        next
    }

    // return Doc
    pub fn value(&self, key: String) -> String{
        let slice = unsafe {
            //let value = CBLResultSet_ValueAtIndex(self.rs, 1);
            let value = CBLResultSet_ValueForKey(self.rs, to_ptr(key));
            let fl_type = FLValue_GetType(value);
            match fl_type  {
                6 => {
                    let fl_slice = FLValue_ToJSON(value); // TODO regarder FLValue_ToJSON5
                    CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(fl_slice.buf as *const u8, fl_slice.size+1)).to_bytes()
                },
                _ => {
                    let fl_slice = FLValue_AsString(value);
                    CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(fl_slice.buf as *const u8, fl_slice.size+1)).to_bytes()
                },
            }
        };
        str::from_utf8(slice).unwrap().to_string()
    }

}

struct Replicator{
    replicator: *mut CBLReplicator,
}

impl Replicator{

    pub fn new(database: Database) -> Self{
        let replicator = unsafe {
            let endpoint = unsafe { CBLEndpoint_NewWithURL(to_ptr("ws://127.0.0.1:4984/mydb".to_string())) };
            /*
            kCBLReplicatorTypePushAndPull = 0,    ///< Bidirectional; both push and pull
            kCBLReplicatorTypePush,               ///< Pushing changes to the target
            kCBLReplicatorTypePull                ///< Pulling changes from the target
            */
            let replicatorType: CBLReplicatorType = 0;
            let config = CBLReplicatorConfiguration {
                database: database.db,
                endpoint: endpoint,
                replicatorType: replicatorType,
                continuous: false,
                authenticator: std::mem::zeroed(),
                pinnedServerCertificate: std::mem::uninitialized(),
                headers: std::mem::uninitialized(),
                channels: std::mem::uninitialized(),
                documentIDs: std::mem::uninitialized(),
                pushFilter: std::mem::uninitialized(),
                pullFilter: std::mem::uninitialized(),
                filterContext: std::mem::uninitialized()
            };
            let mut error: CBLError = std::mem::uninitialized();
            let replicator = CBLReplicator_New(&config, &mut error);
            println!("replicator {:?} - error: {:?}", replicator, error);
            replicator
        };
        Replicator{
            replicator: replicator,
        }
    }

    pub fn start(&self){
        unsafe{ CBLReplicator_Start(self.replicator) };
    }
}

fn main() {
    let database = Database::open("mydb");
    let doc_id = "foo3";
    match database.get_document(doc_id) {
        Some(doc) => {
            println!("1. ================================");
            println!("=====> {:?} - {:?}", doc.id(), doc.jsonify());
            doc.set_value("titi".to_string(), "prop1".to_string());
            println!("2. ================================");
            database.save_document(doc);
            println!("================================ ############ PARTIAL UPDATE");

            let query = database.new_query("SELECT _id AS id, * AS person".to_string());
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
            let duration = Duration::from_secs(15);
            std::thread::sleep(duration);
            println!("================================");
        },
        None => {},
    };


    /*
    let database = Database::open("mydb");
    let doc_id = "foo";
    let document = match database.get_document(doc_id){
        Some(doc) => doc,
        None => database.create_document(doc_id)
    };
    println!("Document ID: {:?}", document.id());

    let person = self::model::Person {
        prop1: "toto".to_string(),
        prop2: "val2".to_string(),
        prop3: "val3".to_string(),
    };
    document.fill(serde_json::to_string_pretty(&fse).unwrap());
    database.save_document(document);

    match database.get_document(doc_id){
        Some(doc) => {
            println!("Document ID: {:?}", doc.id());
            let mut fse: model::FSE = serde_json::from_str(doc.jsonify().as_str()).unwrap();
            fse.prop2 = "val4".to_string();
            doc.fill(serde_json::to_string_pretty(&fse).unwrap());
            database.save_document(doc);

            let final_doc = database.get_document(doc_id).unwrap();
            println!("-- RESULT -- Document ID: {:?} - {:?}", final_doc.id(), final_doc.jsonify());

            println!("================================");
            //let query = database.new_query("SELECT _id, id, prop1 AS person WHERE prop1='val1'".to_string());
            let query = database.new_query("SELECT _id AS id, * AS person".to_string());
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
            let duration = Duration::from_secs(15);
            std::thread::sleep(duration);
            println!("================================");
        },
        None => {}
    };
    */

}
