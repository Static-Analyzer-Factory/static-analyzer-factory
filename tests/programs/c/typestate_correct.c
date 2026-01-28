// Typestate E2E: correct file usage → zero error findings.
#include <stdio.h>

void correct_usage(void) {
    FILE *fp = fopen("data.txt", "r");
    fread(NULL, 1, 1, fp);
    fclose(fp);  // Properly closed.
}
