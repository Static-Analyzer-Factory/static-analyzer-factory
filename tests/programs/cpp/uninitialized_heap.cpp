// CWE-908: Use of Uninitialized Resource (Heap)
// A struct is allocated on the heap with 'new' but its field is never
// initialized. The field is then read, producing an undefined value.
//
// Expected finding: s->value read before initialization
#include <cstdio>

struct Sensor {
    int value;
    int status;
};

int main() {
    Sensor *s = new Sensor;                  // SOURCE: heap alloc, fields uninitialized
    printf("reading: %d\n", s->value);       // SINK: use of uninitialized field
    delete s;
    return 0;
}
