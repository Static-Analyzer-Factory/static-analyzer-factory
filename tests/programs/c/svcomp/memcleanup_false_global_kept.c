// valid-memcleanup FALSE: the block is still allocated when main returns. It is
// reachable through a global, so valid-memtrack holds; valid-memcleanup does not.
#include <stdlib.h>

int *keep;

int main(void) {
  keep = malloc(sizeof(int));
  if (!keep) {
    return 0;
  }
  *keep = 1;
  return 0;
}
