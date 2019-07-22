extern "C" {
    pub fn CBL_Release(document: *const CBLDocument);
}

pub use bindings::*;

mod bindings;
