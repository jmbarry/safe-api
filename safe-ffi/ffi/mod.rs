// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#![allow(clippy::missing_safety_doc)]

mod constants;
mod errors;
/// Fetch API
pub mod fetch;
pub mod ffi_structs;
pub mod files;
pub mod helpers;
pub mod ipc;
pub mod keys;
pub mod logging;
pub mod nrs;
pub mod wallet;
pub mod xorurl;

use errors::Result;
use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use helpers::from_c_str_to_str_option;
use safe_api::Safe;
#[cfg(feature = "mock-network")]
use safe_authenticator_ffi::auth_is_mock;
use safe_core::config_handler;
use std::{
    ffi::{CString, OsStr},
    os::raw::{c_char, c_void},
};

#[no_mangle]
pub unsafe extern "C" fn auth_app(
    app_id: *const c_char,
    app_name: *const c_char,
    app_vendor: *const c_char,
    endpoint: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        auth_response: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let app_id = String::clone_from_repr_c(app_id)?;
        let app_name = String::clone_from_repr_c(app_name)?;
        let app_vendor = String::clone_from_repr_c(app_vendor)?;
        let endpoint = from_c_str_to_str_option(endpoint);
        let auth_response =
            async_std::task::block_on(Safe::auth_app(&app_id, &app_name, &app_vendor, endpoint))?;
        let auth_response = CString::new(auth_response)?;
        o_cb(user_data.0, FFI_RESULT_OK, auth_response.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn connect_app(
    app_id: *const c_char,
    auth_credentials: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, app: *mut Safe),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let app_id = String::clone_from_repr_c(app_id)?;
        let auth_cred = from_c_str_to_str_option(auth_credentials);
        let mut safe = Safe::default();
        async_std::task::block_on(safe.connect(&app_id, auth_cred))?;
        o_cb(user_data.0, FFI_RESULT_OK, Box::into_raw(Box::new(safe)));
        Ok(())
    })
}

/// Sets the path from which the `safe_core.config` file will be read.
#[no_mangle]
pub unsafe extern "C" fn app_set_config_dir_path(
    new_path: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_> {
        let new_path = String::clone_from_repr_c(new_path)?;
        config_handler::set_config_dir_path(OsStr::new(&new_path));
        o_cb(user_data, FFI_RESULT_OK);
        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn app_is_mock() -> bool {
    cfg!(feature = "mock-network")
}

// Don't remove this. This is needed so that build doesn't strip auth_ffi symbols.
#[no_mangle]
#[cfg(feature = "mock-network")]
pub extern "C" fn get_mock_auth() -> bool {
    let _ = auth_is_mock();
    cfg!(feature = "mock-network")
}

#[test]
#[cfg(feature = "mock-network")]
fn test_mock_build() {
    assert_eq!(app_is_mock(), true);
}

// Test mock detection when not compiled against mock-routing.
#[test]
#[cfg(not(feature = "mock-network"))]
fn test_not_mock_build() {
    assert_eq!(app_is_mock(), false);
}
