// CWE-120: Buffer Overflow (Classic)
//
// This program allocates a small buffer with malloc(), then computes
// a pointer past the buffer's end using pointer arithmetic and passes
// it to puts(). This out-of-bounds pointer access is a classic heap
// buffer overflow.
//
// SAF detects this by tracing the malloc return value through pointer
// arithmetic to the puts() argument, identifying that the pointer
// may reference out-of-bounds memory.

#include <stdlib.h>
#include <stdio.h>

int main(void) {
    // Allocate a small buffer (16 bytes)
    char *buf = (char *)malloc(16);

    // BUG: Pointer arithmetic goes past the allocation boundary
    char *oob_ptr = buf + 256;

    // BUG: Accessing out-of-bounds heap memory
    puts(oob_ptr);

    free(buf);
    return 0;
}
