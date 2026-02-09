// CWE-416: Use After Free
//
// This program allocates memory with malloc(), frees it, then
// dereferences the freed pointer. This is undefined behavior and
// can lead to code execution or data corruption.
//
// SAF detects this pattern by tracing the malloc return value
// to both the free() call and subsequent dereferences, identifying
// that the pointer is used after being freed.

#include <stdlib.h>

int main(void) {
    // Allocate heap memory
    int *ptr = (int *)malloc(sizeof(int));
    *ptr = 42;

    // Free the memory — ptr is now dangling
    free(ptr);

    // BUG: Dereference after free — undefined behavior
    int value = *ptr;

    return value;
}
