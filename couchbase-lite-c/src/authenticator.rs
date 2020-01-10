
use ffi;

use crate::to_ptr;

/// Wrapper around `CBLAuthenticator`.
///
/// Used to authenticate a `Replicator` on a (remote) endpoint.
pub struct Authenticator {
    pub (crate) authenticator: *mut ffi::CBLAuthenticator,
}

impl Authenticator {
    /// Create a new authenticator using the HTTP 'basic' authentication method.
    pub fn new_basic(user: String, pwd: String) -> Self {
        let authenticator = unsafe { ffi::CBLAuth_NewBasic(to_ptr(user), to_ptr(pwd)) };
        Self { authenticator }
    }
}