use ffi;

use crate::errors::init_error;
use crate::errors::CouchbaseLiteError;
use crate::resultset::ResultSet;
use std::mem;

pub struct Query {
    pub query: *mut ffi::CBLQuery,
}

impl Query {
    pub fn execute(&self) -> Result<ResultSet, CouchbaseLiteError> {
        let mut error = init_error();
        let rs = unsafe { ffi::CBLQuery_Execute(self.query, &mut error) };
        if error.code == 0 {
            Ok(ResultSet { rs })
        } else {
            Err(CouchbaseLiteError::CannotExecuteQuery(error))
        }
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        unsafe { ffi::CBL_Release( mem::transmute::<*mut ffi::CBLQuery, *mut ffi::CBLRefCounted>(self.query)) };
    }
}
