#include <stdint.h>
#include <assert.h>
#include <stdio.h>
#include <demes_forward.h>

/*
 * Doesn't do error handling.
 * We should have a goto if status != 0
 */
void
iterate_simple_model()
{
    const char* yaml = "time_units: generations\n\
demes:\n\
 - name: A\n\
   epochs:\n\
   - start_size: 100\n\
     end_time: 50\n\
   - start_size: 200\n\
";
    OpaqueForwardGraph* graph = forward_graph_allocate();
    int32_t status;
    const double* model_time;
    const double* parental_deme_sizes;

    status = forward_graph_initialize_from_yaml(yaml, 100.0, graph);
    assert(!forward_graph_is_error_state(graph));
    assert(status == 0);

    status = forward_graph_initialize_time_iteration(graph);
    assert(status == 0);

    for (model_time = forward_graph_iterate_time(graph, &status);
         status == 0 && model_time != NULL;
         model_time = forward_graph_iterate_time(graph, &status))
        {
            /* Update the internal state of the model to model_time */
            status = forward_graph_update_state(*model_time, graph);
            assert(!forward_graph_is_error_state(graph));
            assert(status == 0);
            parental_deme_sizes = forward_graph_parental_deme_sizes(graph, &status);
            assert(status == 0);
            assert(parental_deme_sizes != NULL);
            fprintf(stdout, "%lf %lf\n", *model_time, parental_deme_sizes[0]);
        }
    forward_graph_deallocate(graph);
}

int
main(int argc, char** argv)
{
    iterate_simple_model();
}
