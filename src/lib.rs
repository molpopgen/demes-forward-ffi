use libc::c_char;
use std::ffi::CStr;

pub struct OpaqueForwardGraph {
    graph: Option<demes_forward::ForwardGraph>,
    error: Option<String>,
}

/// # Safety
///
/// `yaml` must be a valid pointer containing valid utf8 data.
pub unsafe extern "C" fn forward_graph_initialize_from_yaml(
    yaml: *const c_char,
    burnin: f64,
) -> *mut OpaqueForwardGraph {
    let yaml = CStr::from_ptr(yaml);
    let yaml = match yaml.to_owned().to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            let b = Box::new(OpaqueForwardGraph {
                graph: None,
                error: Some("could not convert c_char to String".to_string()),
            });
            return Box::into_raw(b);
        }
    };
    let dg = match demes::loads(&yaml) {
        Ok(graph) => graph,
        Err(e) => {
            let b = Box::new(OpaqueForwardGraph {
                graph: None,
                error: Some(format!("{}", e)),
            });
            return Box::into_raw(b);
        }
    };
    let (graph, error) = match demes_forward::ForwardGraph::new(
        dg,
        burnin,
        Some(demes_forward::demes::RoundTimeToInteger::F64),
    ) {
        Ok(graph) => (Some(graph), None),
        Err(e) => (None, Some(format!("{}", e))),
    };
    let b = Box::new(OpaqueForwardGraph { graph, error });
    Box::into_raw(b)
}

/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_is_error_state(graph: *const OpaqueForwardGraph) -> bool {
    (*graph).error.is_some()
}

/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_deallocate(graph: *mut OpaqueForwardGraph) {
    let _ = Box::from_raw(graph);
}

/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_get_error_message(
    graph: *const OpaqueForwardGraph,
) -> *const c_char {
    match &(*graph).error {
        Some(message) => {
            let mref: &str = message;
            let message_cstr = CStr::from_ptr(mref.as_ptr() as *const i8);
            let message_c_char: *const c_char = message_cstr.as_ptr() as *const c_char;
            message_c_char
        }
        None => std::ptr::null(),
    }
}

/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_selfing_rates(
    graph: *const OpaqueForwardGraph,
) -> *const f64 {
    match &(*graph).graph {
        Some(graph) => match graph.selfing_rates() {
            Some(slice) => slice.as_ptr() as *const f64,
            None => std::ptr::null(),
        },
        None => std::ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_alloc_dealloc() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
        let yaml_cstr = CString::new(yaml).unwrap();
        let yaml_c_char: *const c_char = yaml_cstr.as_ptr() as *const c_char;
        let graph = unsafe { forward_graph_initialize_from_yaml(yaml_c_char, 100.0) };
        assert!(unsafe { forward_graph_selfing_rates(graph) }.is_null());
        unsafe { forward_graph_deallocate(graph) };
    }

    #[test]
    fn test_invalid_graph() {
        let yaml = "
time_units: generations
demes:
 - name: A
   start_time: 55
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
        let yaml_cstr = CString::new(yaml).unwrap();
        let yaml_c_char: *const c_char = yaml_cstr.as_ptr() as *const c_char;
        let graph = unsafe { forward_graph_initialize_from_yaml(yaml_c_char, 100.0) };
        assert!(unsafe { forward_graph_is_error_state(graph) });
        let message = unsafe { forward_graph_get_error_message(graph) };
        assert!(!message.is_null());
        let rust_message = unsafe { CStr::from_ptr(message) };
        let rust_message: &str = rust_message.to_str().unwrap();
        assert_eq!(
            rust_message,
            "\"deme A has finite start time but no ancestors\""
        );
        unsafe { forward_graph_deallocate(graph) };
    }
}
