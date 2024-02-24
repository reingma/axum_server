This is a documment where i record results from benchmarks as they evolve.

As features are introduced I run benchmark to see their impacts. State of the
benchmark is also recorded.

Benchmark iterations
1) simple request test ussing apache bench (expect to not be reliable)
    - done with 10000 requests with 100 concurrent.

Tests states:
1) Before intruducing any tracing:
    results for bench 1: avg 0.25sec
2) After intruducing tracing with formatter and instrumented futures, no middleware
    results for bench 1: avg 0.33sec
3) After introducing middleware on requests:
    result for bench 1: avg 0.57sec
