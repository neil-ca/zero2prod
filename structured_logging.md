To ensure that request_id is included in all logs records we should have to:
    * rewrite all upstream components in the request processing pipeline (e.g. actix-web's Logger);
    * change the signature of all downstream functions we are calling from the subscribe handler;
    if they are emitting a log statement, they need to include the request_id, which 
    therefore needs to be passed down as argument.

    It is clear that this approach cannot scale.
    Let's take a step back: what does our code looks like?
    We have an over-arching task (an HTTP request), which is broken down in a set of sub-tasks
    (e.g. parse input, make a query, etc.).
    Each mini task has a contex like the name and email of our endpoint.

## the tracing crate
expands upon logging-style diagnostics by allowing libraries and applications to 
record structured events with additional information about temporality and causality 
unlike a log message, a span in tracing has a beginning and end time, may be entered
and exited by the flow of execution, and may exist whitin a nested tree of similar spans.

