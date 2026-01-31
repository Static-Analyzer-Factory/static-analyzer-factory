#include "aliascheck.h"

char g1;
char g2;

int cond();

void f1(char **p) {
        if (cond()) *p = &g2;
}

char *f2() {
        char *p = &g1;
            f1(&p);
                return p;
}

int main() {
    char *r = f2();
    MAYALIAS(r, &g1);
    MAYALIAS(r, &g2);
    NOALIAS(&g1, &g2);
    return 0;
}
