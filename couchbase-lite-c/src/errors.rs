use ffi;

#[derive(Debug)]
pub enum CouchbaseLiteError {
    CannotOpenDatabase(ffi::CBLError),
    CannotSaveDocument(ffi::CBLError),
    CannotCreateNewQuery(ffi::CBLError),
    CannotFillDocumentFromJson(ffi::CBLError),
}
