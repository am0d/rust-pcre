// Copyright 2013 The rust-pcre authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern mod extra;

use extra::enum_set::{EnumSet};
use std::c_str::{CString};
use std::libc::{c_int, c_char, c_void, c_uchar};
use std::ptr;
use std::result::{Result};

mod native;

pub type fullinfo_field = c_int;
pub struct pcre;
pub type pcre_error = c_int;
pub struct pcre_extra;
pub type study_options = c_int;

pub static PCRE_ERROR_NOMATCH: pcre_error = -1;
pub static PCRE_ERROR_NULL: pcre_error = -2;

pub static PCRE_INFO_OPTIONS: fullinfo_field = 0;
pub static PCRE_INFO_SIZE: fullinfo_field = 1;
pub static PCRE_INFO_CAPTURECOUNT: fullinfo_field = 2;
pub static PCRE_INFO_BACKREFMAX: fullinfo_field = 3;
pub static PCRE_INFO_FIRSTBYTE: fullinfo_field = 4;
pub static PCRE_INFO_FIRSTCHAR: fullinfo_field = 4;
pub static PCRE_INFO_FIRSTTABLE: fullinfo_field = 5;
pub static PCRE_INFO_LASTLITERAL: fullinfo_field = 6;
pub static PCRE_INFO_NAMEENTRYSIZE: fullinfo_field = 7;
pub static PCRE_INFO_NAMECOUNT: fullinfo_field = 8;
pub static PCRE_INFO_NAMETABLE: fullinfo_field = 9;
pub static PCRE_INFO_STUDYSIZE: fullinfo_field = 10;
pub static PCRE_INFO_DEFAULT_TABLES: fullinfo_field = 11;
pub static PCRE_INFO_OKPARTIAL: fullinfo_field = 12;
pub static PCRE_INFO_JCHANGED: fullinfo_field = 13;
pub static PCRE_INFO_HASCRORLF: fullinfo_field = 14;
pub static PCRE_INFO_MINLENGTH: fullinfo_field = 15;
pub static PCRE_INFO_JIT: fullinfo_field = 16;
pub static PCRE_INFO_JITSIZE: fullinfo_field = 17;
pub static PCRE_INFO_MAXLOOKBEHIND: fullinfo_field = 18;
pub static PCRE_INFO_FIRSTCHARACTER: fullinfo_field = 19;
pub static PCRE_INFO_FIRSTCHARACTERFLAGS: fullinfo_field = 20;
pub static PCRE_INFO_REQUIREDCHAR: fullinfo_field = 21;
pub static PCRE_INFO_REQUIREDCHARFLAGS: fullinfo_field = 22;
pub static PCRE_INFO_MATCHLIMIT: fullinfo_field = 23;
pub static PCRE_INFO_RECURSIONLIMIT: fullinfo_field = 24;

pub unsafe fn pcre_compile(pattern: *c_char, options: ::options, tableptr: *c_uchar) -> Result<*mut pcre, (Option<~str>, c_int)> {
    assert!(ptr::is_not_null(pattern));
    let mut err: *c_char = ptr::null();
    let mut erroffset: c_int = 0;
    let code = native::pcre_compile(pattern, options, &mut err, &mut erroffset, tableptr);

    if ptr::is_null(code) {
        // "Otherwise, if  compilation  of  a  pattern fails, pcre_compile() returns
        // NULL, and sets the variable pointed to by errptr to point to a textual
        // error message. This is a static string that is part of the library. You
        // must not try to free it."
        // http://pcre.org/pcre.txt
        let err_cstring = CString::new(err, false);
        //let err_str = match err_cstring.as_str() {
        //    None          => format!("compilation failed at offset {:u}", erroffset as uint),
        //    Some(err_str) => format!("compilation failed at offset {:u}: {:s}", erroffset as uint, err_str)
        //};

        match err_cstring.as_str() {
            None => Err((None, erroffset)),
            Some(err_str) => Err((Some(err_str.to_owned()), erroffset))
        }
    } else {
        assert!(ptr::is_not_null(code));
        assert_eq!(erroffset, 0);

        Ok(code)
    }
}

pub unsafe fn pcre_exec(code: *pcre, extra: *pcre_extra, subject: *c_char, length: c_int, startoffset: c_int, options: ::options, ovector: *mut c_int, ovecsize: c_int) -> c_int {
    assert!(ptr::is_not_null(code));
    assert!(ovecsize >= 0 && ovecsize % 3 == 0);
    let rc = native::pcre_exec(code, extra, subject, length, startoffset, options, ovector, ovecsize);
    if rc == PCRE_ERROR_NOMATCH {
        return -1;
    } else if rc < 0 && rc != PCRE_ERROR_NULL {
        fail!("pcre_exec");
    }

    rc
}

pub unsafe fn pcre_free(ptr: *mut c_void) {
    native::pcre_free(ptr);
}

pub unsafe fn pcre_free_study(extra: *mut pcre_extra) {
    native::pcre_free_study(extra);
}

pub unsafe fn pcre_fullinfo(code: *pcre, extra: *pcre_extra, what: fullinfo_field, where: *mut c_void) {
    assert!(ptr::is_not_null(code));
    let rc = native::pcre_fullinfo(code, extra, what, where);
    if rc < 0 && rc != PCRE_ERROR_NULL {
        fail!("pcre_fullinfo");
    }
}

pub unsafe fn pcre_refcount(code: *mut ::detail::pcre, adjust: c_int) -> c_int {
    assert!(ptr::is_not_null(code));
    let curr_refcount = native::pcre_refcount(code, 0);
    if curr_refcount + adjust < 0 {
        fail!("refcount underflow");
    } else if curr_refcount + adjust > 65535 {
        fail!("refcount overflow");
    }
    native::pcre_refcount(code, adjust)
}

pub unsafe fn pcre_study(code: *::detail::pcre, options: &EnumSet<::StudyOption>) -> *mut ::detail::pcre_extra {
    assert!(ptr::is_not_null(code));
    let converted_options = options.iter().fold(0, |converted_options, option| converted_options | (option as study_options));
    let mut err: *c_char = ptr::null();
    let extra = native::pcre_study(code, converted_options, &mut err);
    // "The third argument for pcre_study() is a pointer for an error message. If
    // studying succeeds (even if no data is returned), the variable it points to is
    // set to NULL. Otherwise it is set to point to a textual error message. This is
    // a static string that is part of the library. You must not try to free it."
    // http://pcre.org/pcre.txt
    if ptr::is_not_null(err) {
        let err_cstring = CString::new(err, false);
        match err_cstring.as_str() {
            None          => error!("pcre_study() failed"),
            Some(err_str) => error!("pcre_study() failed: {}", err_str)
        }
        fail!("pcre_study");
    }
    assert!(ptr::is_null(err));

    extra
}

pub fn pcre_version() -> ~str {
    let version_cstring = unsafe { CString::new(native::pcre_version(), false) };
    version_cstring.as_str().unwrap().to_owned()
}
