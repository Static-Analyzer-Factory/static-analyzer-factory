// valid-memsafety TRUE (safe): in-bounds access, freed once. ASan never traps,
// so the R5 confirmer never confirms a violation -> unknown (SAF never emits
// `true`, and never a false alarm on a safe program).
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
