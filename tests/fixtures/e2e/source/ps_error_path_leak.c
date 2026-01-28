// Path-sensitive test: error path resource handling
// fopen returns NULL on error → fclose called on success path only
// Path-insensitive may report leak on error path.
// Path-sensitive recognizes error path returns early.
#include <stdio.h>
#include <stdlib.h>

int process_file(const char *path) {
    FILE *f = fopen(path, "r");
    if (f == NULL) {
        // Error path: no file to close
        return -1;
    }
    // Success path: use and close
    int ch = fgetc(f);
    fclose(f);
    return ch;
}

int main() {
    return process_file("test.txt");
}
