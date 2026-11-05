// valid-memcleanup TRUE: on a failed allocation the program spins forever
// (never terminates, so nothing is left allocated at a termination).
#include <stdlib.h>

int main(void) {
  int *a = malloc(sizeof(int));
  if (!a)
    return 0;
  int *b = malloc(sizeof(int));
  if (!b) {
    while (1) {
    }
  }
  free(b);
  free(a);
  return 0;
}
