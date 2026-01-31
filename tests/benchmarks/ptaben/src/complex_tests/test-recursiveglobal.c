#include "aliascheck.h"

char *p;

char accessA(unsigned i) {
  return *(p+i);
}


void recursion(unsigned i) {
    if (accessA(i) > 0) return;
    recursion(i++);
}
int main () {
    unsigned i = 0;
    unsigned a[10];
    p = (char *)a;
    recursion(0);

    MAYALIAS(p, (void*)a);
}
