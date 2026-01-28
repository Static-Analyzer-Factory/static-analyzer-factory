// Typestate E2E: double fclose → error state.
#include <stdio.h>

void double_close(void) {
    FILE *fp = fopen("data.txt", "w");
    fprintf(fp, "hello\n");
    fclose(fp);
    fclose(fp);  // Bug: double-close → error state.
}
