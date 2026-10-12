// valid-memsafety TRUE (safe): in-bounds access, freed once. ASan never traps,
// so the R5 confirmer never confirms a violation; the engine's memory-safety
// goal (plan 222 §2s) then proves it -> true (and never a false alarm).
#include <stdlib.h>

int main(void) {
    int *a = (int *)malloc(4 * sizeof(int));
    if (!a) return 0;
    a[0] = 1;
    a[3] = 2;
    int r = a[0] + a[3];
    free(a);
    return r;
}
