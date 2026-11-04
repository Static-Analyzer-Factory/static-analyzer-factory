// valid-memcleanup FALSE: reach_error() aborts the program while the block is
// still allocated. An abort is a termination for valid-memcleanup (the shape of
// list-ext-properties/simple-ext-memcleanup in the 2027 tasks).
extern void __assert_fail(const char *, const char *, unsigned int, const char *);
void reach_error() { __assert_fail("0", "memcleanup_false_error_leaves_block.c", 5, "reach_error"); }
#include <stdlib.h>

int main(void) {
  int *p = malloc(sizeof(int));
  if (!p) {
    return 0;
  }
  *p = 0;
  if (*p == 0) {
    reach_error();
  }
  free(p);
  return 0;
}
