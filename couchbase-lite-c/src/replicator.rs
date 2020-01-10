

use crate::authenticator::Authenticator;
use crate::database::Database;
use crate::to_ptr;
use ffi;

use crate::errors::init_error;
use crate::errors::CouchbaseLiteError;
use std::mem;

pub struct Replicator {
    replicator: *mut ffi::CBLReplicator,
}

impl Replicator {
    pub fn new(database: Database, target_url: String) -> Result<Self, CouchbaseLiteError> {
        let mut error = init_error();
        let replicator = unsafe {
            let endpoint = ffi::CBLEndpoint_NewWithURL(to_ptr(target_url));
            /*
            CBLReplicatorTypePushAndPull = 0,    ///< Bidirectional; both push and pull
            CBLReplicatorTypePush,               ///< Pushing changes to the target
            CBLReplicatorTypePull                ///< Pulling changes from the target
            */
            let replicator_type: ffi::CBLReplicatorType = 0;
            let config = ffi::CBLReplicatorConfiguration {
                database: database.db,
                endpoint: endpoint,
                replicatorType: replicator_type,
                continuous: false,
                authenticator: std::ptr::null_mut(),
                pinnedServerCertificate: ffi::FLSlice {
                    buf: std::ptr::null(),
                    size: 0,
                },
                headers: std::ptr::null(),
                channels: std::ptr::null(),
                documentIDs: std::ptr::null(),
                pushFilter: None,
                pullFilter: None,
                filterContext: std::ptr::null_mut(),
            };
            let r = ffi::CBLReplicator_New(&config, &mut error);
            r
        };
        if error.code == 0 {
            Ok(Replicator { replicator: replicator })
        } else {
            Err(CouchbaseLiteError::CannotCreateNewReplicator(error))
        }
    }

    pub fn new_with_auth(database: Database, target_url: String, auth: Authenticator) -> Result<Self, CouchbaseLiteError> {
        let mut error = init_error();
        let replicator = unsafe {
            let endpoint = ffi::CBLEndpoint_NewWithURL(to_ptr(target_url));
            /*
            CBLReplicatorTypePushAndPull = 0,    ///< Bidirectional; both push and pull
            CBLReplicatorTypePush,               ///< Pushing changes to the target
            CBLReplicatorTypePull                ///< Pulling changes from the target
            */
            let replicator_type: ffi::CBLReplicatorType = 0;
            let config = ffi::CBLReplicatorConfiguration {
                database: database.db,
                endpoint,
                replicatorType: replicator_type,
                continuous: false,
                authenticator: auth.authenticator,
                pinnedServerCertificate: ffi::FLSlice {
                    buf: std::ptr::null(),
                    size: 0,
                },
                headers: std::ptr::null(),
                channels: std::ptr::null(),
                documentIDs: std::ptr::null(),
                pushFilter: None,
                pullFilter: None,
                filterContext: std::ptr::null_mut(),
            };
            ffi::CBLReplicator_New(&config, &mut error)
        };
        if error.code == 0 {
            Ok(Replicator { replicator })
        } else {
            Err(CouchbaseLiteError::CannotCreateNewReplicator(error))
        }
    }

    pub fn start(&self) {
        unsafe { ffi::CBLReplicator_ResetCheckpoint(self.replicator) };
        unsafe { ffi::CBLReplicator_Start(self.replicator) };
    }

    pub fn stop(&self) {
        unsafe { ffi::CBLReplicator_Stop(self.replicator) };
    }
}

impl Drop for Replicator {
    fn drop(&mut self) {
        unsafe { ffi::CBL_Release( mem::transmute::<*mut ffi::CBLReplicator, *mut ffi::CBLRefCounted>(self.replicator)) };
    }
}