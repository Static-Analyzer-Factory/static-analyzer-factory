// CWE-22: Path Traversal
// User-controlled filename from argv is passed directly to fopen().
//
// Expected finding: argv[1] -> fopen() (taint flow)
#include <stdio.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    const char *filename = argv[1];   // SOURCE: user-controlled path
    FILE *f = fopen(filename, "r");   // SINK: file open
    if (f) fclose(f);
    return 0;
}
