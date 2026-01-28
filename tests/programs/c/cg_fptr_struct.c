// CG refinement E2E: Function pointer stored in struct field.
// PTA resolves p->handle(data) to dangerous_handler.
#include <stdlib.h>

typedef void (*handler_fn)(const char *);

struct Plugin {
    handler_fn handle;
    const char *name;
};

void dangerous_handler(const char *s) { system(s); }

void invoke_plugin(struct Plugin *p, const char *data) {
    p->handle(data);  // indirect via struct field
}

int main(void) {
    struct Plugin p;
    p.handle = dangerous_handler;
    p.name = "danger";
    const char *input = getenv("INPUT");
    invoke_plugin(&p, input);
    return 0;
}
