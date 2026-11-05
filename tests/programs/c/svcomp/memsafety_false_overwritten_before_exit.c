// valid-memsafety FALSE (valid-memtrack): the only pointer to the first block
// is overwritten before exit.
#include <stdlib.h>

int main(void) {
  int *a = malloc(sizeof(int));
  if (!a)
    return 0;
  a = malloc(sizeof(int));
  exit(0);
}
