// Typestate E2E: fopen without fclose → file leak (non-accepting at exit).
#include <stdio.h>

void leak_file(void) {
    FILE *fp = fopen("data.txt", "r");
    fread(NULL, 1, 1, fp);
    // Missing fclose(fp) — should report non-accepting (opened) at exit.
}
