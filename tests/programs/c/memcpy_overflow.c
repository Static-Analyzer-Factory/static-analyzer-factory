// Test for memcpy-style buffer overflow detection (CWE-121, CWE-122).
//
// This tests SAF's ability to detect buffer overflows at memcpy/memmove
// call sites rather than just GEP-based array indexing.

#include <string.h>
#include <stdlib.h>

// BAD: Allocate 10 bytes but copy 40 bytes (10 * sizeof(int))
void memcpy_overflow_bad(void) {
    int *data = (int *)malloc(10);  // Only 10 bytes allocated
    int source[10] = {0};
    // OVERFLOW: copying 40 bytes to 10-byte buffer
    memcpy(data, source, 10 * sizeof(int));
    free(data);
}

// GOOD: Allocate proper size
void memcpy_overflow_good(void) {
    int *data = (int *)malloc(10 * sizeof(int));  // 40 bytes allocated
    int source[10] = {0};
    // SAFE: copying 40 bytes to 40-byte buffer
    memcpy(data, source, 10 * sizeof(int));
    free(data);
}

// BAD: Stack buffer with memmove overflow
void memmove_overflow_bad(void) {
    char dest[16];
    char src[32] = "hello world this is a long text";
    // OVERFLOW: copying 32 bytes to 16-byte buffer
    memmove(dest, src, sizeof(src));
}

// GOOD: Stack buffer with proper memmove
void memmove_overflow_good(void) {
    char dest[32];
    char src[32] = "hello world this is a long text";
    // SAFE: copying 32 bytes to 32-byte buffer
    memmove(dest, src, sizeof(src));
}

// BAD: memset overflow
void memset_overflow_bad(void) {
    char buffer[10];
    // OVERFLOW: setting 20 bytes in 10-byte buffer
    memset(buffer, 0, 20);
}

// GOOD: memset proper size
void memset_overflow_good(void) {
    char buffer[20];
    // SAFE: setting 20 bytes in 20-byte buffer
    memset(buffer, 0, 20);
}

int main(void) {
    memcpy_overflow_bad();
    memcpy_overflow_good();
    memmove_overflow_bad();
    memmove_overflow_good();
    memset_overflow_bad();
    memset_overflow_good();
    return 0;
}
