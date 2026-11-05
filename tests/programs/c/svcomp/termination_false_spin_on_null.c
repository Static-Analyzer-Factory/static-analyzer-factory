// termination: never TRUE. A failed allocation spins forever.
#include <stdlib.h>

int main(void) {
  int *a = malloc(sizeof(int));
  if (!a) {
    while (1) {
    }
  }
  free(a);
  return 0;
}
