// valid-memsafety TRUE: `exit` on a failed allocation while the local `a`
// still points to the first block (C locals stay in scope at exit).
#include <stdlib.h>

int main(void) {
  int *a = malloc(sizeof(int));
  if (!a)
    exit(1);
  int *b = malloc(sizeof(int));
  if (!b)
    exit(1);
  *a = 1;
  *b = 2;
  free(b);
  free(a);
  return 0;
}
