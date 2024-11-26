[1mdiff --git a/src/lib.rs b/src/lib.rs[m
[1mindex b73e2d8..9545353 100644[m
[1m--- a/src/lib.rs[m
[1m+++ b/src/lib.rs[m
[36m@@ -19,19 +19,6 @@[m [mlazy_static! {[m
   static ref RO_ACCESS_REPORT_CALLBACK: Mutex<Option<ROAccessReportCallback>> = Mutex::new(None);[m
 }[m
 [m
[31m-#[no_mangle][m
[31m-pub extern "C" fn get_last_error() -> *const c_char {[m
[31m-  let error = LAST_ERROR.lock().unwrap();[m
[31m-  match &*error {[m
[31m-    Some(err) => CString::new(err.clone()).unwrap().into_raw(),[m
[31m-    None => ptr::null(),[m
[31m-  }[m
[31m-}[m
[31m-[m
[31m-fn set_last_error(err: &str) {[m
[31m-  *LAST_ERROR.lock().unwrap() = Some(err.to_string());[m
[31m-}[m
[31m-[m
 // Opaque pointer to represent `LlrpClient` in C[m
 pub struct LlrpClientWrapper(LlrpClient);[m
 [m
[36m@@ -355,4 +342,17 @@[m [mpub extern "C" fn free_string(string_ptr: *mut c_char) -> i32 {[m
     set_last_error("Null string pointer");[m
     return -1;[m
   }[m
[32m+[m[32m}[m
[32m+[m
[32m+[m[32m#[no_mangle][m
[32m+[m[32mpub extern "C" fn get_last_error() -> *const c_char {[m
[32m+[m[32m  let error = LAST_ERROR.lock().unwrap();[m
[32m+[m[32m  match &*error {[m
[32m+[m[32m    Some(err) => CString::new(err.clone()).unwrap().into_raw(),[m
[32m+[m[32m    None => ptr::null(),[m
[32m+[m[32m  }[m
[32m+[m[32m}[m
[32m+[m
[32m+[m[32mfn set_last_error(err: &str) {[m
[32m+[m[32m  *LAST_ERROR.lock().unwrap() = Some(err.to_string());[m
 }[m
\ No newline at end of file[m
