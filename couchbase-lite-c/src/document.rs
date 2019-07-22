use ffi;
use crate::to_ptr;
use crate::to_string;
use std::os::raw::c_void;

pub struct Document/*<T>*/{
    pub doc: *mut ffi::CBLDocument,
    db: Option<*mut ffi::CBLDatabase>,
    //properties: T
}

impl Document{
    pub fn new(id: String) -> Self{
        let doc =  unsafe { ffi::CBLDocument_New(to_ptr(id)) };
        Document{
            doc: doc,
            db: None,
        }

    }

    pub fn from_raw(db: *mut ffi::CBLDatabase, doc: *mut ffi::CBLDocument) -> Self{
        println!("Document::from_raw {:?}", doc);
        Document{
            db: Some(db),
            doc: doc,
        }
    }

    pub fn id(&self) -> String{
        println!("====> 1. {:?} ", self.doc);
        let doc_id = unsafe { ffi::CBLDocument_ID(self.doc) };
        println!("====> {:?}", doc_id);
        to_string(doc_id)
    }

    // TODO add generic T: Serialize
    // putProperties
    pub fn fill(&self, json: String) -> bool {
        let mut error: ffi::CBLError = unsafe { std::mem::uninitialized() };
        let json_string = to_ptr(json);
        let status = unsafe { ffi::CBLDocument_SetPropertiesAsJSON(self.doc, json_string, &mut error) };
        println!("jsonify {:?} - error: {:?}", status, error);
        status
    }

    // getProperties
    pub fn jsonify(&self) -> String {
        let json: *mut ::std::os::raw::c_char = unsafe { ffi::CBLDocument_PropertiesAsJSON(self.doc) };
        to_string(json)
    }

    //TODO implement Deref and call unsafe { ffi:CBRelease(saved) };

    pub fn set_value(&self, value: String, for_key: String) {
        unsafe {
            let properties = ffi::CBLDocument_MutableProperties(self.doc);

            let key_str = to_ptr(for_key.clone());
            let key = ffi::FLString {
                buf: key_str as *const c_void,
                //size: libc::strlen(key_str),
                size: for_key.len()
            };
//            println!("//////////////////// {:?} - {:?}", for_key.len(), libc::strlen(key_str));

            let fl_value = ffi::FLDict_Get(properties, key);
            let fl_string = ffi::FLValue_AsString(fl_value);
            println!("//////////////////// {:?} - {:?}", properties, key);
            let fl_slot = ffi::FLMutableDict_Set(properties, key);

            let val = ffi::FLString {
                buf: to_ptr(value.clone()) as *const c_void,
                size: value.len()
            };
            ffi::FLSlot_SetString(fl_slot, val);

        }

    }

    // TODO add patch or update method and use Fleece JSON Delta API

    pub fn sequence(&self) -> u64{
         unsafe{ ffi::CBLDocument_Sequence(self.doc) }
    }

}

#[cfg(test)]
mod tests {
    use crate::Database;
    use crate::Document;

    #[test]
    fn new_document() {
        let doc_id = String::from("foo");
        let doc = Document::new(doc_id.clone());
        assert_eq!(doc_id, doc.id());
        assert_eq!(0, doc.sequence());
        assert_eq!("{}", doc.jsonify());
        assert_eq!(unsafe{ffi::CBLDocument_MutableProperties(doc.doc)} as *const ffi::_FLDict, unsafe{ffi::CBLDocument_Properties(doc.doc)});
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


}

