use ffi;

use crate::resultset::ResultSet;
use crate::init_error;

pub struct Query {
    pub query: *mut ffi::CBLQuery,
}

impl Query {
    pub fn execute(&self) -> ResultSet {
        let mut error = init_error();
        let rs = unsafe { ffi::CBLQuery_Execute(self.query, &mut error) };
        // FIXME handle errors
        ResultSet { rs: rs }
    }
}
