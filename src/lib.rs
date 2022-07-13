use libc::c_char;
use std::ffi::CStr;

pub struct OpaqueForwardGraph {
    graph: Option<demes_forward::ForwardGraph>,
    error: Option<String>,
}

impl OpaqueForwardGraph {
    fn update(&mut self, graph: Option<demes_forward::ForwardGraph>, error: Option<String>) {
        self.graph = graph;
        self.error = error.map(|e| {
            e.chars()
                .filter(|c| c.is_ascii() && c != &'"')
                .collect::<String>()
        });
    }
}

/// Allocate an [`OpaqueForwardGraph`]
///
/// # Panics
///
/// This function will panic if the pointer allocation fails.
///
/// # Safety
///
/// The pointer is returned by leaking a [`Box`].
/// The pointer is managed by rust and is freed by [`forward_graph_deallocate`].
#[no_mangle]
pub extern "C" fn forward_graph_allocate() -> *mut OpaqueForwardGraph {
    Box::into_raw(Box::new(OpaqueForwardGraph {
        graph: None,
        error: None,
    }))
}

/// # Safety
///
/// `yaml` must be a valid pointer containing valid utf8 data.
#[no_mangle]
pub unsafe extern "C" fn forward_graph_initialize_from_yaml(
    yaml: *const c_char,
    burnin: f64,
    graph: *mut OpaqueForwardGraph,
) {
    if yaml.is_null() {
        (*graph).update(None, Some("could not convert c_char to String".to_string()));
        return;
    }
    let yaml = CStr::from_ptr(yaml);
    let yaml = match yaml.to_owned().to_str() {
        Ok(s) => s.to_string(),
        Err(e) => {
            (*graph).update(None, Some(format!("{}", e)));
            return;
        }
    };
    let dg = match demes::loads(&yaml) {
        Ok(graph) => graph,
        Err(e) => {
            (*graph).update(None, Some(format!("{}", e)));

            return;
        }
    };
    match demes_forward::ForwardGraph::new(
        dg,
        burnin,
        Some(demes_forward::demes::RoundTimeToInteger::F64),
    ) {
        Ok(fgraph) => (*graph).update(Some(fgraph), None),
        Err(e) => (*graph).update(None, Some(format!("{}", e))),
    }
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

/// Pointer to first element of selfing rates array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
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

/// Pointer to first element of cloning rates array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_cloning_rates(
    graph: *const OpaqueForwardGraph,
) -> *const f64 {
    match &(*graph).graph {
        Some(graph) => match graph.cloning_rates() {
            Some(slice) => slice.as_ptr() as *const f64,
            None => std::ptr::null(),
        },
        None => std::ptr::null(),
    }
}

/// Pointer to first element of parental deme size array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_parental_deme_sizes(
    graph: *const OpaqueForwardGraph,
) -> *const f64 {
    match &(*graph).graph {
        Some(graph) => match graph.parental_deme_sizes() {
            Some(slice) => slice.as_ptr() as *const f64,
            None => std::ptr::null(),
        },
        None => std::ptr::null(),
    }
}

/// Pointer to first element of offspring deme size array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_offspring_deme_sizes(
    graph: *const OpaqueForwardGraph,
) -> *const f64 {
    match &(*graph).graph {
        Some(graph) => match graph.offspring_deme_sizes() {
            Some(slice) => slice.as_ptr() as *const f64,
            None => std::ptr::null(),
        },
        None => std::ptr::null(),
    }
}

/// Check if there are any extant offspring demes.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_any_extant_offspring_demes(
    graph: *const OpaqueForwardGraph,
) -> bool {
    match &(*graph).graph {
        Some(graph) => graph.any_extant_offspring_demes(),
        None => false,
    }
}

/// Check if there are any extant parental demes.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_any_extant_parent_demes(
    graph: *const OpaqueForwardGraph,
) -> bool {
    match &(*graph).graph {
        Some(graph) => graph.any_extant_parental_demes(),
        None => false,
    }
}

/// Get the total number of demes in the model
///
/// # Returns
///
/// [`isize`] > 0 if the graph is not in an error state.
/// Returns `-1` otherwise.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_number_of_demes(graph: *const OpaqueForwardGraph) -> isize {
    match &(*graph).graph {
        Some(graph) => graph.num_demes_in_model() as isize,
        None => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    struct GraphHolder {
        graph: *mut OpaqueForwardGraph,
    }

    impl GraphHolder {
        fn new() -> Self {
            Self {
                graph: forward_graph_allocate(),
            }
        }

        fn as_mut_ptr(&mut self) -> *mut OpaqueForwardGraph {
            self.graph
        }

        fn as_ptr(&mut self) -> *const OpaqueForwardGraph {
            self.graph
        }

        fn init_with_yaml(&mut self, burnin: f64, yaml: &str) {
            let yaml_cstr = CString::new(yaml).unwrap();
            let yaml_c_char: *const c_char = yaml_cstr.as_ptr() as *const c_char;
            unsafe { forward_graph_initialize_from_yaml(yaml_c_char, burnin, self.as_mut_ptr()) };
        }
    }

    impl Drop for GraphHolder {
        fn drop(&mut self) {
            unsafe { forward_graph_deallocate(self.as_mut_ptr()) };
        }
    }

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
        let mut graph = GraphHolder::new();
        graph.init_with_yaml(100.0, yaml);
        assert!(unsafe { forward_graph_selfing_rates(graph.as_mut_ptr()) }.is_null());
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
        let mut graph = GraphHolder::new();
        graph.init_with_yaml(100.0, yaml);
        assert!(unsafe { forward_graph_is_error_state(graph.as_ptr()) });
        let message = unsafe { forward_graph_get_error_message(graph.as_ptr()) };
        assert!(!message.is_null());
        let rust_message = unsafe { CStr::from_ptr(message) };
        let rust_message: &str = rust_message.to_str().unwrap();
        assert_eq!(
            rust_message,
            "deme A has finite start time but no ancestors"
        );
    }

    #[test]
    fn test_empty_graph() {
        let yaml = "";
        let mut graph = GraphHolder::new();
        graph.init_with_yaml(100.0, yaml);
        assert!(unsafe { forward_graph_is_error_state(graph.as_ptr()) });
    }

    #[test]
    fn test_null_graph() {
        let yaml: *const c_char = std::ptr::null();
        let graph = forward_graph_allocate();
        unsafe { forward_graph_initialize_from_yaml(yaml, 100.0, graph) };
        assert!(unsafe { forward_graph_is_error_state(graph) });
        unsafe { forward_graph_deallocate(graph) };
    }

    #[test]
    fn number_of_demes_in_model() {
        {
            let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
            let mut graph = GraphHolder::new();
            graph.init_with_yaml(100.0, yaml);
            let num_demes = unsafe { forward_graph_number_of_demes(graph.as_ptr()) };
            assert_eq!(num_demes, 1);
        }
    }

    #[test]
    fn getters_are_none_when_state_not_updated() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
        let mut graph = GraphHolder::new();
        graph.init_with_yaml(100.0, yaml);
        assert!(unsafe { forward_graph_selfing_rates(graph.as_ptr()) }.is_null());
        assert!(unsafe { forward_graph_cloning_rates(graph.as_ptr()) }.is_null());
        assert!(unsafe { forward_graph_parental_deme_sizes(graph.as_ptr()) }.is_null());
        assert!(unsafe { forward_graph_offspring_deme_sizes(graph.as_ptr()) }.is_null());
        assert!(!unsafe { forward_graph_any_extant_offspring_demes(graph.as_ptr()) });
        assert!(!unsafe { forward_graph_any_extant_parent_demes(graph.as_ptr()) });
    }
}
