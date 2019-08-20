use crate::database::Database;
use crate::to_ptr;
use ffi;
use std::string::ToString;

use crate::errors::init_error;
use crate::errors::CouchbaseLiteError;
use std::mem::MaybeUninit;

pub struct Replicator {
    replicator: *mut ffi::CBLReplicator,
}

impl Replicator {
    pub fn new(database: Database) -> Result<Self, CouchbaseLiteError> {
        let mut error = init_error();
        let replicator = unsafe {
            // TODO extract URL
            let endpoint = ffi::CBLEndpoint_NewWithURL(to_ptr("ws://127.0.0.1:4984/mydb".to_string()));
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
                authenticator: std::mem::zeroed(),
                pinnedServerCertificate: std::mem::uninitialized(), //MaybeUninit::uninit().assume_init(),
                headers: std::mem::uninitialized(),                 //MaybeUninit::uninit().assume_init(),
                channels: std::mem::uninitialized(),                //MaybeUninit::uninit().assume_init(),
                documentIDs: std::mem::uninitialized(),             //MaybeUninit::uninit().assume_init(),
                pushFilter: std::mem::uninitialized(),              //MaybeUninit::uninit().assume_init(),
                pullFilter: std::mem::uninitialized(),              //MaybeUninit::uninit().assume_init(),
                filterContext: std::mem::uninitialized(),           //MaybeUninit::uninit().assume_init(),
            };
            println!("================================ CBLReplicatorConfiguration : {:?}", config);
            let r = ffi::CBLReplicator_New(&config, &mut error);
            r
        };
        if error.code == 0 {
            Ok(Replicator { replicator: replicator })
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
