use crate::to_ptr;
use crate::to_string;
use ffi;

use crate::document::Document;
use crate::errors::init_error;
use crate::errors::CouchbaseLiteError;
use crate::query::Query;
use core::ptr;

#[derive(Clone, Debug)]
pub struct Database {
    pub db: *mut ffi::CBLDatabase,
}

impl Database {
    pub fn open(directory: String, name: &str) -> Result<Self, CouchbaseLiteError> {
        let mut error = init_error();
        let database_name = to_ptr(name.to_string());
        let CBLDatabase_Create: ffi::CBLDatabaseFlags = 1;
        // No encryption (default)
        let CBLEncryptionNone: ffi::CBLEncryptionAlgorithm = 0;
        let config = ffi::CBLDatabaseConfiguration {
            directory: to_ptr(directory),
            flags: CBLDatabase_Create,
            encryptionKey: ffi::CBLEncryptionKey {
                algorithm: CBLEncryptionNone,
                bytes: [0; 32usize],
            },
        };
        let db = unsafe { ffi::CBLDatabase_Open(database_name, &config, &mut error) };
        println!("open database error: {:?}", error);
        if error.code == 0 {
            Ok(Database { db: db })
        } else {
            Err(CouchbaseLiteError::CannotOpenDatabase(error))
        }
    }

    pub fn create_document(&self, id: String) -> Document {
        let doc_id = to_ptr(id);
        let doc = unsafe { ffi::CBLDocument_New(doc_id) };
        Document::from_raw(self.db, doc)
    }

    pub fn get_document(&self, id: String) -> Option<Document> {
        let doc_id = to_ptr(id.to_string());
        let mut doc = unsafe { ffi::CBLDatabase_GetMutableDocument(self.db, doc_id) };
        if doc.is_null() {
            None
        } else {
            Some(Document::from_raw(self.db, doc))
        }
    }

    pub fn save_document(&self, document: Document) -> Result<Document, CouchbaseLiteError> {
        let mut error = init_error();
        let CBLConcurrencyControlLastWriteWins: ffi::CBLConcurrencyControl = 0;
        let CBLConcurrencyControlFailOnConflict: ffi::CBLConcurrencyControl = 1;
        let json: *mut ::std::os::raw::c_char = unsafe { ffi::CBLDocument_PropertiesAsJSON(document.doc) };
        //println!("BEFORE save document doc: {:?}", to_string(json));
        let saved: *const ffi::CBLDocument =
            unsafe { ffi::CBLDatabase_SaveDocument(self.db, document.doc, CBLConcurrencyControlLastWriteWins, &mut error) };
        //println!("save document error: {:?}", error);
        if error.code == 0 && saved != ptr::null() {
            let json: *mut ::std::os::raw::c_char = unsafe { ffi::CBLDocument_PropertiesAsJSON(saved) };
            //println!("AFTER save document saved_doc: {:?}", to_string(json));
            let doc = unsafe { ffi::CBLDocument_MutableCopy(saved) };
            Ok(Document::from_raw(self.db, doc))
        } else {
            Err(CouchbaseLiteError::CannotSaveDocument(error))
        }
    }

    pub fn new_query(&self, n1ql_query: String) -> Result<Query, CouchbaseLiteError> {
        let n1ql_query_language: ffi::CBLQueryLanguage = 1;
        let query_string = to_ptr(n1ql_query);
        let mut outErrorPos: ::std::os::raw::c_int = 0;
        let mut error = init_error();
        let query = unsafe { ffi::CBLQuery_New(self.db, n1ql_query_language, query_string, &mut outErrorPos, &mut error) };
        if error.code == 0 {
            Ok(Query { query: query })
        } else {
            Err(CouchbaseLiteError::CannotCreateNewQuery(error))
        }
    }

    pub fn get_name(&self) -> String {
        let name = unsafe { ffi::CBLDatabase_Name(self.db) };
        to_string(name)
    }

    pub fn get_path(&self) -> String {
        let path = unsafe { ffi::CBLDatabase_Path(self.db) };
        to_string(path)
    }

    pub fn count(&self) -> u64 {
        unsafe { ffi::CBLDatabase_Count(self.db) }
    }

    pub fn in_batch(&self, unit: &Fn() -> ()) -> Result<(), CouchbaseLiteError> {
        let mut error = init_error();
        let status = unsafe { ffi::CBLDatabase_BeginBatch(self.db, &mut error) };
        if error.code == 0 && status {
            (unit)();
            let status = unsafe { ffi::CBLDatabase_EndBatch(self.db, &mut error) };
            if error.code == 0 && status {
                return Ok(());
            }
        }
        return Err(CouchbaseLiteError::ErrorInBatch(error));
    }

    pub fn close(&self) -> Result<(),CouchbaseLiteError> {
        let mut error = init_error();
        let status = unsafe { ffi::CBLDatabase_Close(self.db, &mut error) };
        if error.code == 0 && status {
            Ok(())
        } else {
            Err(CouchbaseLiteError::CannotCloseDatabase(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;
    use crate::Document;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;
    use uuid::Uuid;

    fn test_dir() -> String {
        let uuid = Uuid::new_v4().to_string();
        let dir = format!("/tmp/testdb/{}", uuid);
        match fs::create_dir_all(dir.clone()) {
            Ok(_) => {}
            Err(e) => panic!("Cannot create database directory: {:?}", e),
        };
        dir
    }

    #[test]
    fn test_open_database() {
        let database_name = String::from("testdb");
        let test_dir = test_dir();
        let database = Database::open(test_dir.clone(), &database_name.clone());
        assert_eq!(true, database.is_ok());
        let database = database.unwrap();
        assert_eq!(database_name, database.get_name());
        assert_eq!(format!("{}/{}.cblite2/", test_dir, database_name), database.get_path());
        assert_eq!(0, database.count());
    }

    fn open_database() -> Database {
        let database_name = String::from("testdb");
        let database = Database::open(test_dir(), &database_name);
        assert_eq!(true, database.is_ok());
        database.unwrap()
    }

    #[test]
    fn save_empty_document() {
        let database = open_database();
        let doc_id = String::from("foo");
        {
            let doc = Document::new(doc_id.clone());
            let saved = database.save_document(doc);
            assert_eq!(true, saved.is_ok());
            let saved = saved.unwrap();
            assert_eq!(doc_id, saved.id());
            assert_eq!(1, saved.sequence());
            assert_eq!("{}", saved.jsonify());
        }
        {
            let doc = database.get_document(doc_id.clone());
            assert_eq!(true, doc.is_some());
            let doc = doc.unwrap();
            assert_eq!(doc_id, doc.id());
            assert_eq!(1, doc.sequence());
            assert_eq!("{}", doc.jsonify());
        }
    }

    #[test]
    fn save_document_with_property() {
        let database = open_database();
        let doc_id = String::from("foo");
        {
            let doc = Document::new(doc_id.clone());
            doc.set_value(String::from("Howdy!"), String::from("greeting"));
            assert_eq!("{\"greeting\":\"Howdy!\"}", doc.jsonify());

            let saved = database.save_document(doc);
            assert_eq!(true, saved.is_ok());
            let saved = saved.unwrap();
            assert_eq!(doc_id, saved.id());
            assert_eq!(1, saved.sequence());
            assert_eq!("{\"greeting\":\"Howdy!\"}", saved.jsonify());
        }
        {
            let doc = database.get_document(doc_id.clone());
            assert_eq!(true, doc.is_some());
            let doc = doc.unwrap();
            assert_eq!(doc_id, doc.id());
            assert_eq!(1, doc.sequence());
            assert_eq!("{\"greeting\":\"Howdy!\"}", doc.jsonify());
        }
    }

    #[test]
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
        assert_eq!(true, saved.is_ok());
        let saved = saved.unwrap();
        assert_eq!(doc_id, saved.id());
        assert_eq!(1, saved.sequence());
        assert_eq!("{\"first_name\":\"James\",\"last_name\":\"Bomb\"}", saved.jsonify());
    }

    #[test]
    fn update_existing_document_with_existing_property() {
        let database = open_database();
        let doc_id = String::from("foo");
        {
            let doc = Document::new(doc_id.clone());
            doc.set_value(String::from("val1"), String::from("prop1"));
            assert_eq!("{\"prop1\":\"val1\"}", doc.jsonify());

            let saved = database.save_document(doc);
            assert_eq!(true, saved.is_ok());
            let saved = saved.unwrap();
            assert_eq!(doc_id, saved.id());
            assert_eq!(1, saved.sequence());
            assert_eq!("{\"prop1\":\"val1\"}", saved.jsonify());

            saved.set_value(String::from("val2"), String::from("prop1"));
            database.save_document(saved);
        }
        {
            let doc = database.get_document(doc_id.clone());
            assert_eq!(true, doc.is_some());
            let doc = doc.unwrap();
            assert_eq!(doc_id, doc.id());
            assert_eq!(2, doc.sequence());
            assert_eq!("{\"prop1\":\"val2\"}", doc.jsonify());
        }
    }

    #[test]
    fn update_existing_document_with_new_property() {
        let database = open_database();
        let doc_id = String::from("foo");
        database.in_batch(&|| {
            {
                // Create document
                let doc = Document::new(doc_id.clone());
                doc.set_value(String::from("val1"), String::from("prop1"));
                assert_eq!("{\"prop1\":\"val1\"}", doc.jsonify());
                let saved = database.save_document(doc);
                assert_eq!(true, saved.is_ok());
                let saved = saved.unwrap();
                assert_eq!(doc_id, saved.id());
                assert_eq!(1, saved.sequence());
                assert_eq!("{\"prop1\":\"val1\"}", saved.jsonify());
            }
            {
                // Update Document
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                // Add new property
                doc.set_value(String::from("val2"), String::from("prop2"));
                database.save_document(doc);
            }
            {
                // Verify Document
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                assert_eq!(doc_id, doc.id());
                assert_eq!(2, doc.sequence());
                assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\"}", doc.jsonify());
            }
        });
    }

    #[test]
    fn multiple_update_document() {
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Struct1 {
            pub prop1: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub prop2: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub prop3: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub prop4: Option<String>,
        }
        let doc_id = String::from("foo");
        let database = open_database();
        {
            // Create Document
            let doc = Document::new(doc_id.clone());
            let data = Struct1 {
                prop1: "val1".to_string(),
                prop2: None,
                prop3: None,
                prop4: None,
            };
            //doc.set_value(String::from("val1"), String::from("prop1"));
            doc.fill(serde_json::to_string_pretty(&data).unwrap());
            assert_eq!("{\"prop1\":\"val1\"}", doc.jsonify());
            let saved = database.save_document(doc);
            assert_eq!(true, saved.is_ok());
            let saved = saved.unwrap();
            assert_eq!(doc_id, saved.id());
            assert_eq!(1, saved.sequence());
            assert_eq!("{\"prop1\":\"val1\"}", saved.jsonify());

        }
        {
            // First Update
            database.in_batch(&|| {
                // Update Document
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                // Add new property
                let data = Struct1 {
                    prop1: "val1".to_string(),
                    prop2: Some("val2".to_string()),
                    prop3: None,
                    prop4: None,
                };
                doc.fill(serde_json::to_string_pretty(&data).unwrap());
                //doc.set_value(String::from("val2"), String::from("prop2"));
                let saved = database.save_document(doc);
                assert_eq!(true, saved.is_ok());
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                assert_eq!(doc_id, doc.id());
                assert_eq!(2, doc.sequence());
                assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\"}", doc.jsonify());
            });
        }
        {
            // Second Update
            database.in_batch(&|| {
                // Update Document
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                // Add new property
                let data = Struct1 {
                    prop1: "val1".to_string(),
                    prop2: Some("val2".to_string()),
                    prop3: Some("val3".to_string()),
                    prop4: None,
                };
                doc.fill(serde_json::to_string_pretty(&data).unwrap());
                //doc.set_value(String::from("val3"), String::from("prop3"));
                assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\",\"prop3\":\"val3\"}", doc.jsonify());
                let saved = database.save_document(doc);
                assert_eq!(true, saved.is_ok());
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                assert_eq!(doc_id, doc.id());
                assert_eq!(3, doc.sequence());
                assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\",\"prop3\":\"val3\"}", doc.jsonify());
            });
        }
        {
            // Second Update
            database.in_batch(&|| {
                // Update Document
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                // Add new property
                let data = Struct1 {
                    prop1: "val1".to_string(),
                    prop2: Some("val2".to_string()),
                    prop3: Some("val3".to_string()),
                    prop4: Some("val4".to_string()),
                };
                doc.fill(serde_json::to_string_pretty(&data).unwrap());
                //doc.set_value(String::from("val4"), String::from("prop4"));
                assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\",\"prop3\":\"val3\",\"prop4\":\"val4\"}", doc.jsonify());
                let saved = database.save_document(doc);
                assert_eq!(true, saved.is_ok());
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                assert_eq!(doc_id, doc.id());
                assert_eq!(4, doc.sequence());
                assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\",\"prop3\":\"val3\",\"prop4\":\"val4\"}", doc.jsonify());
            });
        }

    }

}
