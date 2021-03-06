extern crate libsqlite3_sys as ffi;
extern crate libc;

use std::ffi::{CString, CStr};
use std::io::{stderr, Write};
use std::{ptr, str};

use result::*;
use result::Error::DatabaseError;

pub struct RawConnection {
    pub internal_connection: *mut ffi::sqlite3,
}

impl RawConnection {
    pub fn establish(database_url: &str) -> ConnectionResult<Self> {
        let mut conn_pointer = ptr::null_mut();
        let database_url = try!(CString::new(database_url));
        let connection_status = unsafe {
            ffi::sqlite3_open(database_url.as_ptr(), &mut conn_pointer)
        };

        match connection_status {
            ffi::SQLITE_OK => Ok(RawConnection {
                internal_connection: conn_pointer,
            }),
            err_code => {
                let message = super::error_message(err_code);
                Err(ConnectionError::BadConnection(message.into()))
            }
        }
    }

    pub fn exec(&self, query: &str) -> QueryResult<()> {
        let mut err_msg = ptr::null_mut();
        let query = try!(CString::new(query));
        let callback_fn = None;
        let callback_arg = ptr::null_mut();
        unsafe {
            ffi::sqlite3_exec(
                self.internal_connection,
                query.as_ptr(),
                callback_fn,
                callback_arg,
                &mut err_msg,
            );
        }

        if !err_msg.is_null() {
            let msg = convert_to_string_and_free(err_msg);
            Err(DatabaseError(msg))
        } else {
            Ok(())
        }
    }

    pub fn rows_affected_by_last_query(&self) -> usize {
        unsafe { ffi::sqlite3_changes(self.internal_connection) as usize }
    }
}

impl Drop for RawConnection {
    fn drop(&mut self) {
        let close_result = unsafe { ffi::sqlite3_close(self.internal_connection) };
        if close_result != ffi::SQLITE_OK {
            let error_message = super::error_message(close_result);
            write!(stderr(), "Error closing SQLite connection: {}", error_message).unwrap();
        }
    }
}

fn convert_to_string_and_free(err_msg: *const libc::c_char) -> String {
    let msg = unsafe {
        let bytes = CStr::from_ptr(err_msg).to_bytes();
        str::from_utf8_unchecked(bytes).into()
    };
    unsafe { ffi::sqlite3_free(err_msg as *mut libc::c_void) };
    msg
}
