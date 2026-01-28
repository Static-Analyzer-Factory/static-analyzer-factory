#include <stdlib.h>

// Simulate a custom resource (e.g., database connection)
typedef struct {
    int handle;
    char *name;
} Resource;

Resource *acquire_resource() {
    Resource *r = (Resource *)malloc(sizeof(Resource));
    r->handle = 1;
    r->name = (char *)malloc(64);
    return r;
}

void release_resource(Resource *r) {
    free(r->name);
    free(r);
}

void process() {
    Resource *r = acquire_resource();
    r->handle = 42;

    // BUG: r is never released via release_resource()
    // (inner mallocs are also leaked)
}

int main() {
    process();
    return 0;
}
