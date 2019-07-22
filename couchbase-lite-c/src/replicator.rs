use crate::database::Database;
use crate::to_ptr;
use ffi;
use std::string::ToString;

pub struct Replicator {
    replicator: *mut ffi::CBLReplicator,
}

impl Replicator {
    pub fn new(database: Database) -> Self {
        let replicator = unsafe {
            // TODO extract URL
            let endpoint = unsafe { ffi::CBLEndpoint_NewWithURL(to_ptr("ws://127.0.0.1:4984/mydb".to_string())) };
            /*
            CBLReplicatorTypePushAndPull = 0,    ///< Bidirectional; both push and pull
            CBLReplicatorTypePush,               ///< Pushing changes to the target
            CBLReplicatorTypePull                ///< Pulling changes from the target
            */
            let replicatorType: ffi::CBLReplicatorType = 0;
            let config = ffi::CBLReplicatorConfiguration {
                database: database.db,
                endpoint: endpoint,
                replicatorType: replicatorType,
                continuous: false,
                authenticator: std::mem::zeroed(),
                pinnedServerCertificate: std::mem::uninitialized(),
                headers: std::mem::uninitialized(),
                channels: std::mem::uninitialized(),
                documentIDs: std::mem::uninitialized(),
                pushFilter: std::mem::uninitialized(),
                pullFilter: std::mem::uninitialized(),
                filterContext: std::mem::uninitialized(),
            };
            let mut error: ffi::CBLError = std::mem::uninitialized();
            let replicator = ffi::CBLReplicator_New(&config, &mut error);
            // FIXME handle errors
            replicator
        };
        Replicator { replicator: replicator }
    }

    pub fn start(&self) {
        unsafe { ffi::CBLReplicator_Start(self.replicator) };
    }
}
