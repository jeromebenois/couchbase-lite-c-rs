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
                authenticator: MaybeUninit::zeroed().assume_init(),
                pinnedServerCertificate: MaybeUninit::uninit().assume_init(),
                headers: MaybeUninit::uninit().assume_init(),
                channels: MaybeUninit::uninit().assume_init(),
                documentIDs: MaybeUninit::uninit().assume_init(),
                pushFilter: MaybeUninit::uninit().assume_init(),
                pullFilter: MaybeUninit::uninit().assume_init(),
                filterContext: MaybeUninit::uninit().assume_init(),
            };
            ffi::CBLReplicator_New(&config, &mut error)
        };
        if error.code == 0 {
            Ok(Replicator { replicator: replicator })
        } else {
            Err(CouchbaseLiteError::CannotCreateNewReplicator(error))
        }
    }

    pub fn start(&self) {
        unsafe { ffi::CBLReplicator_Start(self.replicator) };
    }
}
