use crate::errors::init_error;
use crate::errors::CouchbaseLiteError;
use crate::to_ptr;
use crate::to_string;
use ffi;
use std::os::raw::c_void;

// TODO add generic T: Serialize
//TODO implement Deref and call unsafe { ffi:CBRelease(saved) };
pub struct Document /*<T>*/ {
    pub doc: *mut ffi::CBLDocument,
    db: Option<*mut ffi::CBLDatabase>,
    //properties: T
}

impl Document {
    pub fn new(id: String) -> Self {
        let doc = unsafe { ffi::CBLDocument_New(to_ptr(id)) };
        Document { doc: doc, db: None }
    }

    pub fn from_raw(db: *mut ffi::CBLDatabase, doc: *mut ffi::CBLDocument) -> Self {
        Document { db: Some(db), doc: doc }
    }

    pub fn id(&self) -> String {
        let doc_id = unsafe { ffi::CBLDocument_ID(self.doc) };
        to_string(doc_id)
    }

    pub fn fill(&self, json: String) -> Result<bool, CouchbaseLiteError> {
        let mut error = init_error();
        let json_string = to_ptr(json);
        let status = unsafe { ffi::CBLDocument_SetPropertiesAsJSON(self.doc, json_string, &mut error) };
        println!("jsonify {:?} - error: {:?}", status, error);
        if error.code == 0 {
            Ok(status)
        } else {
            Err(CouchbaseLiteError::CannotFillDocumentFromJson(error))
        }
    }

    pub fn jsonify(&self) -> String {
        let json: *mut ::std::os::raw::c_char = unsafe { ffi::CBLDocument_PropertiesAsJSON(self.doc) };
        to_string(json)
    }

    pub fn set_value(&self, value: String, for_key: String) {
        unsafe {
            let properties = ffi::CBLDocument_MutableProperties(self.doc);

            let key_str = to_ptr(for_key.clone());
            let key = ffi::FLString {
                buf: key_str as *const c_void,
                size: for_key.len(),
            };
            let fl_value = ffi::FLDict_Get(properties, key);
            let fl_string = ffi::FLValue_AsString(fl_value);
            let fl_slot = ffi::FLMutableDict_Set(properties, key);

            let val = ffi::FLString {
                buf: to_ptr(value.clone()) as *const c_void,
                size: value.len(),
            };
            ffi::FLSlot_SetString(fl_slot, val);
        }
    }

    pub fn sequence(&self) -> u64 {
        unsafe { ffi::CBLDocument_Sequence(self.doc) }
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;
    use crate::Document;
    use serde::{Deserialize, Serialize};

    #[test]
    fn new_document() {
        let doc_id = String::from("foo");
        let doc = Document::new(doc_id.clone());
        assert_eq!(doc_id, doc.id());
        assert_eq!(0, doc.sequence());
        assert_eq!("{}", doc.jsonify());
        assert_eq!(unsafe { ffi::CBLDocument_MutableProperties(doc.doc) } as *const ffi::_FLDict, unsafe {
            ffi::CBLDocument_Properties(doc.doc)
        });
    }

    #[test]
    fn add_new_property_in_document() {
        let doc_id = String::from("foo");
        let doc = Document::new(doc_id);
        doc.set_value(String::from("val1"), String::from("prop1"));
        assert_eq!("{\"prop1\":\"val1\"}", doc.jsonify());

        // Add new property
        doc.set_value(String::from("val2"), String::from("prop2"));
        assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\"}", doc.jsonify());
    }

    #[test]
    fn fill_document_from_json_string() {
        let doc_id = String::from("foo");
        let doc = Document::new(doc_id);
        let status = doc.fill(String::from("{\"prop1\":\"val1\",\"prop2\":\"val2\"}"));
        assert_eq!(true, status.is_ok());
        let status = status.unwrap();
        assert_eq!(true, status);
        assert_eq!("{\"prop1\":\"val1\",\"prop2\":\"val2\"}", doc.jsonify());
    }

    #[test]
    fn fill_document_from_json_struct() {
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Person {
            pub first_name: String,
            pub last_name: String,
        }
        let person = Person {
            first_name: "James".to_string(),
            last_name: "Bomb".to_string(),
        };
        let doc_id = String::from("foo");
        let doc = Document::new(doc_id);
        let status = doc.fill(serde_json::to_string_pretty(&person).unwrap());
        assert_eq!(true, status.is_ok());
        let status = status.unwrap();
        assert_eq!(true, status);
        assert_eq!("{\"first_name\":\"James\",\"last_name\":\"Bomb\"}", doc.jsonify());
    }

}