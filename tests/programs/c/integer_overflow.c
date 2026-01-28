// CWE-190: Integer Overflow leading to undersized allocation
// Unchecked multiplication of user-supplied size and count may wrap,
// causing malloc to allocate a much smaller buffer than expected.
//
// Expected finding: unchecked size * count -> malloc (integer overflow)
#include <stdlib.h>
#include <string.h>

void *alloc_array(unsigned int count, unsigned int size) {
    unsigned int total = count * size;       // SOURCE: unchecked multiply (may wrap)
    void *buf = malloc(total);               // SINK: undersized allocation
    if (buf) {
        memset(buf, 0, count * size);        // writes past allocation on overflow
    }
    return buf;
}

int main(void) {
    void *p = alloc_array(0x40000001, 4);    // 0x40000001 * 4 wraps to 4
    free(p);
    return 0;
}
