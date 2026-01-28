// Typestate E2E: fread after fclose → error state.
#include <stdio.h>

void use_after_close(void) {
    FILE *fp = fopen("data.txt", "r");
    fclose(fp);
    fread(NULL, 1, 1, fp);  // Bug: use-after-close → error state.
}
