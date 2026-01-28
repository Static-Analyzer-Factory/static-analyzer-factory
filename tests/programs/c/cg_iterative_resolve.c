// CG refinement E2E: Two-level indirection requiring 2+ iterations.
// Iteration 1 discovers f = trampoline, iteration 2 resolves trampoline's body.
#include <stdlib.h>

typedef void (*sink_fn)(const char *);

void final_sink(const char *s) { system(s); }

void trampoline(sink_fn fn, const char *data) {
    fn(data);  // 2nd indirect call resolved in iteration 2
}

typedef void (*dispatch_fn)(sink_fn, const char *);

void setup(dispatch_fn *out) {
    *out = trampoline;  // store function pointer
}

int main(void) {
    dispatch_fn f;
    setup(&f);
    const char *input = getenv("CMD");
    f(final_sink, input);
    return 0;
}
