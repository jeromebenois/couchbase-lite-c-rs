use crate::to_ptr;
use crate::to_string;
use ffi;

use crate::document::Document;
use crate::errors::init_error;
use crate::errors::CouchbaseLiteError;
use crate::query::Query;

use core::ptr;
use std::cell::Cell;

#[derive(Clone, Debug)]
pub struct Database {
    pub db: *mut ffi::CBLDatabase,
    open: Cell<bool>,   // this could just be a bool but then we'd have to
                        //incompatibly change close signature to fn close(&mut self)
}

impl Database {
    fn from(db: *mut ffi::CBLDatabase) -> Self {
        Database{ db, open: Cell::new(true) }
    }

    pub fn open(directory: String, name: &str) -> Result<Self, CouchbaseLiteError> {
        let mut error = init_error();
        let database_name = to_ptr(name.to_string());
        let flags: ffi::CBLDatabaseFlags = 1;
        // No encryption (default)
        let encrypt_galgo: ffi::CBLEncryptionAlgorithm = 0;
        let config = ffi::CBLDatabaseConfiguration {
            directory: to_ptr(directory),
            flags,
            encryptionKey: ffi::CBLEncryptionKey {
                algorithm: encrypt_galgo,
                bytes: [0; 32usize],
            },
        };
        let db = unsafe { ffi::CBLDatabase_Open(database_name, &config, &mut error) };
        if error.code == 0 {
            println!("open database status: {:?} (OK)", error);
            Ok(Database::from(db))
        } else {
            println!("open database error: {:?}", error);
            Err(CouchbaseLiteError::CannotOpenDatabase(error))
        }
    }

    pub fn create_index(&self, name: &str, column_expression: &str) -> Result<(), CouchbaseLiteError> {
        let mut error = init_error();
        let index_name = to_ptr(name.to_string());

        let config = ffi::CBLIndexSpec {
            type_: 0,//0 -> kCBLValueIndex -> An index that stores property or expression values, 1 -> kCBLFullTextIndex -> An index of strings, that enables searching for words with `MATCH`
            keyExpressionsJSON: to_ptr(column_expression.to_string()),
            ignoreAccents: false,
            language: ptr::null()
        };
        let result = unsafe { ffi::CBLDatabase_CreateIndex(self.db, index_name, config, &mut error) };
        if result {
            Ok(())
        } else {
            Err(CouchbaseLiteError::CannotOpenDatabase(error))
        }
    }

    /// Creates a new, empty document in memory. It will not be added to a database until saved.
    pub fn create_document(&self, id: String) -> Document {
        let doc_id = to_ptr(id);
        let doc = unsafe { ffi::CBLDocument_New(doc_id) };
        Document::from_raw(self.db, doc)
    }

    /// Fetches a document with given id (if there is one).
    pub fn get_document(&self, id: String) -> Option<Document> {
        let doc_id = to_ptr(id);
        let doc = unsafe { ffi::CBLDatabase_GetMutableDocument(self.db, doc_id) };
        if doc.is_null() {
            None
        } else {
            let (doc, is_empty_doc) = unsafe {
                let dict = ffi::CBLDocument_MutableProperties(doc);
                (doc, ffi::FLDict_IsEmpty(dict))
            };
            if is_empty_doc {
                // document is deleted
                None
            } else {
                Some(Document::from_raw(self.db, doc))
            }
        }
    }

    /// Saves a (mutable) document to the database.
    pub fn save_document(&self, document: Document) -> Result<Document, CouchbaseLiteError> {
        let is_empty_doc = unsafe {
            let dict = ffi::CBLDocument_MutableProperties(document.doc);
            ffi::FLDict_IsEmpty(dict)
        };
        if is_empty_doc {
            Err(CouchbaseLiteError::CannotSaveEmptyDocument)
        } else {
            let mut error = init_error();
            let concurrency_last_write_wins: ffi::CBLConcurrencyControl = 0; // concurrency_fail_on_conflict: ffi::CBLConcurrencyControl = 1
            let _json: *mut ::std::os::raw::c_char = unsafe { ffi::CBLDocument_PropertiesAsJSON(document.doc) };

            //println!("BEFORE save document doc: {:?}", to_string(json));
            let saved: *const ffi::CBLDocument =
                unsafe { ffi::CBLDatabase_SaveDocument(self.db, document.doc, concurrency_last_write_wins, &mut error) };
            //println!("save document error: {:?}", error);
            if error.code == 0 && saved.is_null() {
                let _json: *mut ::std::os::raw::c_char = unsafe { ffi::CBLDocument_PropertiesAsJSON(saved) };
                //println!("AFTER save document saved_doc: {:?}", to_string(json));
                let doc = unsafe { ffi::CBLDocument_MutableCopy(saved) };
                Ok(Document::from_raw(self.db, doc))
            } else {
                Err(CouchbaseLiteError::CannotSaveDocument(error))
            }
        }
    }

    /// Deletes a document from the database. Deletions are replicated.
    ///
    /// Warning You are still responsible for releasing the CBLDocument.
    ///
    /// ### Arguments
    ///
    /// * document The document to delete.
    /// * concurrency Conflict-handling strategy.
    /// * error On failure, the error will be written here.
    ///
    /// ### Return value
    ///
    /// True if the document was deleted, false if an error occurred.
    pub fn delete_document(&self, document: Document) -> Result<bool, CouchbaseLiteError> {
        let mut error = init_error();
        let concurrency_last_write_wins: ffi::CBLConcurrencyControl = 0;
        let deleted = unsafe { ffi::CBLDocument_Delete(document.doc, concurrency_last_write_wins, &mut error) };
        if error.code == 0 {
            Ok(deleted)
        } else {
            Err(CouchbaseLiteError::CannotDeleteDocument(error))
        }
    }

    /// Creates a new query by compiling the input string.
    pub fn new_query(&self, n1ql_query: String) -> Result<Query, CouchbaseLiteError> {
        let n1ql_query_language: ffi::CBLQueryLanguage = 1;
        let query_string = to_ptr(n1ql_query);
        let mut out_error_pos: ::std::os::raw::c_int = 0;
        let mut error = init_error();
        let query = unsafe { ffi::CBLQuery_New(self.db, n1ql_query_language, query_string, &mut out_error_pos, &mut error) };
        if error.code == 0 {
            Ok(Query { query })
        } else {
            Err(CouchbaseLiteError::CannotCreateNewQuery(error))
        }
    }

    /// Returns the database's name.
    pub fn get_name(&self) -> String {
        let name = unsafe { ffi::CBLDatabase_Name(self.db) };
        to_string(name)
    }

    /// Returns the database's full filesystem path.
    pub fn get_path(&self) -> String {
        let path = unsafe { ffi::CBLDatabase_Path(self.db) };
        to_string(path)
    }

    /// Returns the number of documents in the database.
    pub fn count(&self) -> u64 {
        unsafe { ffi::CBLDatabase_Count(self.db) }
    }

    /// Executes an operation as a "batch", similar to a transaction.
    pub fn in_batch(&self, unit: &dyn Fn() -> ()) -> Result<(), CouchbaseLiteError> {
        let mut error = init_error();
        let status = unsafe { ffi::CBLDatabase_BeginBatch(self.db, &mut error) };
        if error.code == 0 && status {
            (unit)();
            let status = unsafe { ffi::CBLDatabase_EndBatch(self.db, &mut error) };
            if error.code == 0 && status {
                return Ok(());
            }
        }
        Err(CouchbaseLiteError::ErrorInBatch(error))
    }

    /// Executes an operation as a "batch", similar to a transaction. The operation function can return a result.
    pub fn in_batch_with_result<T>(&self, unit: &dyn Fn() -> Result<T, CouchbaseLiteError>) -> Result<T, CouchbaseLiteError> {
        let mut error = init_error();
        let status = unsafe { ffi::CBLDatabase_BeginBatch(self.db, &mut error) };
        if error.code == 0 && status {
            let result = (unit)();
            let status = unsafe { ffi::CBLDatabase_EndBatch(self.db, &mut error) };
            if error.code == 0 && status {
                return result;
            }
        }
        Err(CouchbaseLiteError::ErrorInBatch(error))
    }

    pub fn close(&self) -> Result<(), CouchbaseLiteError> {
        let mut error = init_error();
        let status = unsafe { ffi::CBLDatabase_Close(self.db, &mut error) };
        if error.code == 0 && status {
            self.open.set(false);
            Ok(())
        } else {
            Err(CouchbaseLiteError::CannotCloseDatabase(error))
        }
    }

    /// Deletes the (opened) database. After the database is deleted, the database object (self) is closed.
    pub fn delete(&self) -> Result<(), CouchbaseLiteError> {
        let mut error = init_error();
        let status = unsafe { ffi::CBLDatabase_Delete(self.db, &mut error) };
        if error.code == 0 && status {
            self.open.set(false);
            Ok(())
        } else {
            Err(CouchbaseLiteError::CannotDeleteDatabase(error))
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if self.open.get() {
            let _ = self.close();
            self.open.set(false);
        }
        unsafe { ffi::CBL_Release(self.db as *mut ffi::CBLRefCounted) };
    }
}


#[cfg(test)]
mod tests {
    use crate::Database;
    use crate::Document;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::time::Instant;
    use uuid::Uuid;

    use serde_json::json;

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
        let doc = Document::new(doc_id);
        let saved = database.save_document(doc);
        assert_eq!(true, saved.is_err());
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
    fn test_index() {
        let database = open_database();
        for i in 0..10000 {
            let doc = Document::new(format!("id_{}", i));
            doc.set_value(format!("Howdy{}!", i), String::from("greeting"));
            let saved = database.save_document(doc);
            assert!(saved.is_ok());
        }

        let start = Instant::now();
        let query = database.new_query("SELECT _id AS id, _rev as rev, * AS patient WHERE greeting='Howdy1!'".to_string()).unwrap();
        let rs = query.execute().unwrap();

        assert!(rs.has_next());
        assert!(start.elapsed().as_millis() > 10);

        //For columln_expression cf. c4db_createIndex in couchbase-lite-core project
        let created = database.create_index("greeting_index", "[[\".greeting\"]]");
        assert!(created.is_ok());

        let start = Instant::now();
        let rs = query.execute().unwrap();
        assert!(rs.has_next());
        assert!(start.elapsed().as_millis() < 1);

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
        doc.fill(serde_json::to_string_pretty(&person).unwrap()).unwrap();
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
            database.save_document(saved).unwrap();
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
                database.save_document(doc).unwrap();
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
        }).unwrap();
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
            doc.fill(serde_json::to_string_pretty(&data).unwrap()).unwrap();
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
                doc.fill(serde_json::to_string_pretty(&data).unwrap()).unwrap();
                //doc.set_value(String::from("val2"), String::from("prop2"));
                let saved = database.save_document(doc);
                assert_eq!(true, saved.is_ok());
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                assert_eq!(doc_id, doc.id());
                assert_eq!(2, doc.sequence());
                assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\"}", doc.jsonify());
            }).unwrap();
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
                doc.fill(serde_json::to_string_pretty(&data).unwrap()).unwrap();
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
            }).unwrap();
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
                doc.fill(serde_json::to_string_pretty(&data).unwrap()).unwrap();
                //doc.set_value(String::from("val4"), String::from("prop4"));
                assert_eq!(
                    "{\"prop1\":\"val1\",\"prop2\":\"val2\",\"prop3\":\"val3\",\"prop4\":\"val4\"}",
                    doc.jsonify()
                );
                let saved = database.save_document(doc);
                assert_eq!(true, saved.is_ok());
                let doc = database.get_document(doc_id.clone());
                assert_eq!(true, doc.is_some());
                let doc = doc.unwrap();
                assert_eq!(doc_id, doc.id());
                assert_eq!(4, doc.sequence());
                assert_eq!(
                    "{\"prop1\":\"val1\",\"prop2\":\"val2\",\"prop3\":\"val3\",\"prop4\":\"val4\"}",
                    doc.jsonify()
                );
            }).unwrap();
        }
    }

    #[test]
    fn delete_document() {
        let database = open_database();
        let doc_id = String::from("foo");
        {
            let doc = Document::new(doc_id.clone());
            doc.fill(json!({"prop1": "val1"}).to_string()).unwrap();
            let saved = database.save_document(doc);
            assert_eq!(true, saved.is_ok());
            let saved = saved.unwrap();
            assert_eq!(doc_id, saved.id());
            assert_eq!(1, saved.sequence());
            assert_eq!(json!({"prop1": "val1"}).to_string(), saved.jsonify());
        }
        {
            let document = database.get_document(doc_id.clone()).unwrap();
            let deleted = database.delete_document(document);
            assert!(deleted.is_ok());
            assert!(deleted.unwrap());
        }
        {
            let document = database.get_document(doc_id);
            assert!(document.is_none());
        }
    }

    #[test]
    fn delete_document_with_two_sessions() {
        let database_name = String::from("testdb");
        let test_dir = test_dir();
        let doc_id = String::from("foo");
        {
            #[derive(Serialize, Deserialize, Debug)]
            pub struct Struct1 {
                pub prop1: String,
            }
            let database = Database::open(test_dir.clone(), &database_name).unwrap();
            let doc = Document::new(doc_id.clone());
            let data = Struct1 { prop1: "val1".to_string() };
            doc.fill(serde_json::to_string_pretty(&data).unwrap()).unwrap();
            let saved = database.save_document(doc);
            assert_eq!(true, saved.is_ok());
            let saved = saved.unwrap();
            assert_eq!(doc_id, saved.id());
            assert_eq!(1, saved.sequence());
            assert_eq!("{\"prop1\":\"val1\"}", saved.jsonify());
            database.close().unwrap();
        }
        {
            let database = Database::open(test_dir.clone(), &database_name).unwrap();
            let document = database.get_document(doc_id.clone()).unwrap();
            let deleted = database.delete_document(document);
            assert!(deleted.is_ok());
            assert!(deleted.unwrap());
            database.close().unwrap();
        }
        {
            let database = Database::open(test_dir, &database_name).unwrap();
            let document = database.get_document(doc_id);
            assert!(document.is_none());
            database.close().unwrap();
        }
    }

    #[test]
    fn get_existing_document() {
        let database = open_database();
        let doc_id = String::from("foo");
        {
            let doc = Document::new(doc_id.clone());
            doc.fill(json!({"prop1": "val1"}).to_string()).unwrap();
            let saved = database.save_document(doc);
            assert_eq!(true, saved.is_ok());
            let saved = saved.unwrap();
            assert_eq!(doc_id, saved.id());
            assert_eq!(1, saved.sequence());
            assert_eq!(json!({"prop1": "val1"}).to_string(), saved.jsonify());
        }
        {
            let document = database.get_document(doc_id.clone()).unwrap();
            assert_eq!(doc_id, document.id());
        }
    }

    #[test]
    fn get_inexisting_document() {
        let database = open_database();
        let doc_id = String::from("inexisting");
        let document = database.get_document(doc_id);
        assert!(document.is_none());
    }

}
