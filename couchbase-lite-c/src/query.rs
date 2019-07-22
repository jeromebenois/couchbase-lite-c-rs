use ffi;

use crate::resultset::ResultSet;

pub struct Query {
    pub query: *mut ffi::CBLQuery,
}

impl Query {
    pub fn execute(&self) -> ResultSet {
        let mut error: ffi::CBLError = unsafe { std::mem::uninitialized() };
        let rs = unsafe { ffi::CBLQuery_Execute(self.query, &mut error) };
        // FIXME handle errors
        ResultSet { rs: rs }
    }
}
