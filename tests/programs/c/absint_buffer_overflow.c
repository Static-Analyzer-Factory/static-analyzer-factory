// Abstract interpretation test: buffer overflow detection (CWE-120)
// This program has:
// 1. A safe array access within a bounds-checked loop
// 2. An off-by-one overflow past the array end
// 3. A clearly out-of-bounds access

#include <stdlib.h>

// Fixed-size buffer with known bounds
#define BUF_SIZE 10

void safe_access(int *buf) {
    // Safe: loop counter i in [0, 9], BUF_SIZE = 10
    for (int i = 0; i < BUF_SIZE; i++) {
        buf[i] = i * 2;
    }
}

void off_by_one(int *buf) {
    // Bug: loop goes to i <= BUF_SIZE, so buf[10] is OOB
    for (int i = 0; i <= BUF_SIZE; i++) {
        buf[i] = i;
    }
}

int main(void) {
    int buf[BUF_SIZE];
    safe_access(buf);
    off_by_one(buf);
    return buf[0];
}
