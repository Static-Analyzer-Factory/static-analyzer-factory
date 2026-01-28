// CG refinement E2E: Function pointer callback.
// PTA resolves `handler(data)` in dispatch() to dangerous_sink.
#include <stdlib.h>

void dangerous_sink(const char *cmd) { system(cmd); }

void dispatch(void (*handler)(const char *), const char *data) {
    handler(data);  // indirect call resolved via PTA
}

int main(void) {
    const char *input = getenv("USER_CMD");
    dispatch(dangerous_sink, input);
    return 0;
}
