// Path-sensitive test: correlated branch prevents use-after-free
// The `freed` flag guards against use-after-free.
// Path-insensitive: reports UAF (free → *p on SVFG)
// Path-sensitive: filters it (freed=1 → !freed is always false → *p unreachable)
#include <stdlib.h>

void use_value(int x);

int main() {
    int *p = (int *)malloc(sizeof(int));
    if (!p) return 1;
    *p = 10;

    int freed = 0;
    free(p);
    freed = 1;

    if (!freed) {
        // This path is infeasible: freed is always 1 here
        use_value(*p);  // Would be UAF, but path is infeasible
    }
    return 0;
}
