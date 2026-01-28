// Function Pointer Callback
// A function pointer is stored and called indirectly. PTA must resolve
// the pointer to determine which function is actually invoked.
//
// Expected finding: indirect call via fp resolves to process()
#include <stdio.h>

typedef void (*callback_t)(int);

void process(int x) {
    printf("processing: %d\n", x);           // resolved target
}

void invoke(callback_t fp, int val) {
    fp(val);                                  // SINK: indirect call through fn ptr
}

int main(void) {
    callback_t cb = process;                  // SOURCE: function address taken
    invoke(cb, 42);
    return 0;
}
