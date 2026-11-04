// valid-memcleanup TRUE: the program frees its block in its own atexit handler,
// which runs before termination. The native replay's check runs after it, so it
// must not report the block.
#include <stdlib.h>

int *keep;

void cleanup(void) { free(keep); }

int main(void) {
  atexit(cleanup);
  keep = malloc(sizeof(int));
  if (!keep) {
    return 0;
  }
  *keep = 1;
  return 0;
}
