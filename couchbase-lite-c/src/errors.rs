use ffi;

#[derive(Debug)]
pub enum CouchbaseLiteError {
    CannotOpenDatabase(ffi::CBLError),
    CannotCloseDatabase(ffi::CBLError),
    CannotSaveDocument(ffi::CBLError),
    CannotSaveEmptyDocument,
    CannotDeleteDocument(ffi::CBLError),
    CannotCreateNewQuery(ffi::CBLError),
    CannotFillDocumentFromJson(ffi::CBLError),
    CannotCreateNewReplicator(ffi::CBLError),
    CannotExecuteQuery(ffi::CBLError),
    ErrorInBatch(ffi::CBLError),
}

pub fn init_error() -> ffi::CBLError {
    ffi::CBLError {
        domain: 0,
        code: 0,
        internal_info: 0,
    }
}
