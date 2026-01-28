// File descriptor leak: fopen without fclose.
// Expected: file-descriptor-leak checker should find a finding.
#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *f = fopen("/tmp/test.txt", "w");
    if (f) {
        fprintf(f, "hello");
    }
    return 0;
    // LEAK: f is never closed (fclose not called)
}
