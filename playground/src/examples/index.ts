/** Built-in example C programs for the playground. */

export interface Example {
  name: string;
  slug: string;
  description: string;
  source: string;
}

const pointerAlias: Example = {
  name: 'Pointer Aliasing',
  slug: 'pointer_alias',
  description: 'Two pointers to the same struct, demonstrating alias analysis',
  source: `#include <stdlib.h>

struct Point {
    int x;
    int y;
};

int main() {
    struct Point *p = malloc(sizeof(struct Point));
    p->x = 10;
    p->y = 20;

    struct Point *q = p;  // q aliases p
    q->x = 30;

    // Both p->x and q->x are now 30
    int val = p->x + q->y;
    return val;
}`,
};

const indirectCall: Example = {
  name: 'Indirect Call',
  slug: 'indirect_call',
  description: 'Function pointer calls, demonstrating call graph resolution',
  source: `typedef int (*BinOp)(int, int);

int add(int a, int b) { return a + b; }
int mul(int a, int b) { return a * b; }

int apply(BinOp fn, int x, int y) {
    return fn(x, y);  // indirect call
}

int main() {
    BinOp op = add;
    int r1 = apply(op, 3, 4);

    op = mul;
    int r2 = apply(op, 3, 4);

    return r1 + r2;
}`,
};

const structField: Example = {
  name: 'Struct Fields',
  slug: 'struct_field',
  description: 'Linked list with struct fields, demonstrating field-sensitive PTA',
  source: `#include <stdlib.h>

struct Node {
    int value;
    struct Node *next;
};

struct Node *create(int val) {
    struct Node *n = malloc(sizeof(struct Node));
    n->value = val;
    n->next = NULL;
    return n;
}

int main() {
    struct Node *a = create(1);
    struct Node *b = create(2);

    a->next = b;          // a.next -> b
    b->next = create(3);  // b.next -> new node

    int sum = a->value + a->next->value + b->next->value;
    return sum;
}`,
};

const taintFlow: Example = {
  name: 'Taint Flow',
  slug: 'taint_flow',
  description: 'Value flows from input to output, demonstrating value-flow analysis',
  source: `#include <stdio.h>

int source() {
    int x;
    scanf("%d", &x);  // taint source
    return x;
}

void sink(int val) {
    printf("result: %d\\n", val);  // taint sink
}

int transform(int input) {
    return input * 2 + 1;
}

int main() {
    int tainted = source();
    int derived = transform(tainted);
    sink(derived);  // tainted value flows to sink
    return 0;
}`,
};

const complexCFG: Example = {
  name: 'Complex CFG',
  slug: 'complex_cfg',
  description: 'Single function with loops, nested branches, and switch — for CFG exploration',
  source: `int main() {
    int x = 0, y = 100;

    // while-loop with nested if-else chain
    while (x < 10) {
        if (x % 3 == 0) {
            y += x;
        } else if (x % 3 == 1) {
            y -= x;
        } else {
            y *= 2;
        }
        x++;
    }

    // for-loop with early break and continue
    for (int i = 0; i < y; i++) {
        if (i == 42)
            break;
        if (i % 7 == 0)
            continue;
        x += i;
    }

    // switch with fallthrough and default
    switch (x % 5) {
        case 0: y = 1;   break;
        case 1: y = 10;  break;
        case 2: y = 100; break;
        case 3:           // fallthrough
        case 4: y = -1;  break;
        default: y = 0;
    }

    // short-circuit boolean + ternary
    int z = (x > 50 && y > 0) ? x : y;

    return z;
}`,
};

const libraryModeling: Example = {
  name: 'Library Modeling (Specs)',
  slug: 'library_modeling',
  description:
    'Demonstrates how function specs improve PTA precision for malloc/free/strcpy',
  source: `#include <stdlib.h>
#include <string.h>

// Enable "Specs" in the settings bar to see the difference!
//
// With specs: PTA knows malloc() returns a fresh heap object,
// strcpy() copies data, and free() deallocates memory.
// Without specs: library calls are opaque black boxes.

char *duplicate(const char *src) {
    size_t len = strlen(src);
    char *buf = (char *)malloc(len + 1);
    strcpy(buf, src);    // With specs: buf content aliases src
    return buf;           // With specs: return points to malloc site
}

void process(const char *input) {
    char *copy = duplicate(input);
    // With specs: copy -> malloc allocation in duplicate()
    // Without specs: copy's points-to set is empty

    char *other = (char *)malloc(256);
    // With specs: other is a DIFFERENT allocation than copy

    strcpy(other, copy);
    // With specs: other content flows from copy

    free(copy);
    // With specs: copy is deallocated

    // BUG: use-after-free (copy was freed)
    other[0] = copy[0];

    free(other);
}

int main() {
    process("hello world");
    return 0;
}`,
};

const useAfterFree: Example = {
  name: 'Use-After-Free',
  slug: 'use_after_free',
  description: 'Heap buffer accessed after free — classic UAF vulnerability',
  source: `#include <stdlib.h>
#include <string.h>

void process() {
    char *buffer = (char *)malloc(64);
    strcpy(buffer, "Hello, allocated memory!");

    // Free the memory
    free(buffer);

    // BUG: use-after-free — reading freed memory
    char c = buffer[0];

    // BUG: use-after-free — writing to freed memory
    buffer[0] = 'X';
}

int main() {
    process();
    return 0;
}`,
};

const memoryLeak: Example = {
  name: 'Memory Leak',
  slug: 'memory_leak',
  description: 'Heap allocation not freed — CWE-401 memory leak detection',
  source: `#include <stdlib.h>

int *create_buffer(int size) {
    int *p = (int *)malloc(size * sizeof(int));
    p[0] = 42;
    return p;
}

void process() {
    int *buf = create_buffer(10);
    buf[1] = 100;
    // BUG: buf is never freed — memory leak
}

int main() {
    process();
    return 0;
}`,
};

const doubleFree: Example = {
  name: 'Double Free',
  slug: 'double_free',
  description: 'Memory freed twice — CWE-415 double-free detection',
  source: `#include <stdlib.h>

void process() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;

    free(p);

    // BUG: p is freed again
    free(p);
}

int main() {
    process();
    return 0;
}`,
};

const nullDeref: Example = {
  name: 'Null Dereference',
  slug: 'null_deref',
  description: 'Pointer used without null check — CWE-476 null dereference detection',
  source: `#include <stdlib.h>

int process() {
    int *p = (int *)malloc(sizeof(int));
    // BUG: malloc can return NULL, but p is used without check
    *p = 42;
    int val = *p;
    free(p);
    return val;
}

int main() {
    return process();
}`,
};

const fileDescriptorLeak: Example = {
  name: 'File Descriptor Leak',
  slug: 'file_descriptor_leak',
  description: 'File opened but not closed — CWE-775 file descriptor leak detection',
  source: `#include <stdio.h>

void process(const char *path) {
    FILE *f = fopen(path, "r");
    if (!f) return;

    char buf[256];
    fgets(buf, sizeof(buf), f);

    // BUG: f is never closed
}

int main() {
    process("data.txt");
    return 0;
}`,
};

const lockNotReleased: Example = {
  name: 'Lock Not Released',
  slug: 'lock_not_released',
  description: 'Mutex locked but not unlocked — CWE-764 lock safety',
  source: `#include <pthread.h>

pthread_mutex_t mtx;
int shared_data = 0;

void process() {
    pthread_mutex_lock(&mtx);
    shared_data++;
    // BUG: lock is never released
}

int main() {
    pthread_mutex_init(&mtx, 0);
    process();
    pthread_mutex_destroy(&mtx);
    return 0;
}`,
};

const uninitUse: Example = {
  name: 'Uninitialized Use',
  slug: 'uninit_use',
  description: 'Heap memory read before initialization — CWE-908 uninitialized use',
  source: `#include <stdlib.h>

int process() {
    int *p = (int *)malloc(sizeof(int));

    // BUG: reading *p before writing to it
    int val = *p;

    free(p);
    return val;
}

int main() {
    return process();
}`,
};

const stackEscape: Example = {
  name: 'Stack Escape',
  slug: 'stack_escape',
  description: 'Local variable address returned — CWE-562 stack escape detection',
  source: `int *get_value() {
    int x = 42;
    // BUG: returning address of stack variable
    return &x;
}

int main() {
    int *p = get_value();
    return *p;
}`,
};

const genericResourceLeak: Example = {
  name: 'Generic Resource Leak',
  slug: 'generic_resource_leak',
  description: 'Custom resource acquired but not released — CWE-772 resource leak',
  source: `#include <stdlib.h>

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
}`,
};

export const examples: Example[] = [
  pointerAlias,
  indirectCall,
  structField,
  taintFlow,
  complexCFG,
  useAfterFree,
  libraryModeling,
  memoryLeak,
  doubleFree,
  nullDeref,
  fileDescriptorLeak,
  lockNotReleased,
  uninitUse,
  stackEscape,
  genericResourceLeak,
];
