use ffi;

#[derive(Debug)]
pub enum CouchbaseLiteError {
    CannotOpenDatabase(ffi::CBLError),
    CannotSaveDocument(ffi::CBLError),
    CannotCreateNewQuery(ffi::CBLError),
    CannotFillDocumentFromJson(ffi::CBLError),
    CannotCreateNewReplicator(ffi::CBLError),
    CannotExecuteQuery(ffi::CBLError),
}

pub fn init_error() -> ffi::CBLError {
    ffi::CBLError {
        domain: 0,
        code: 0,
        internal_info: 0,
    }
}