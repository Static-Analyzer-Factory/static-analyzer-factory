// unreach-call: never TRUE. Two heap objects' addresses are unrelated: their
// difference may be anything, including 64.
#include <stdlib.h>
extern void reach_error(void);

int main(void) {
  char *p = malloc(16);
  char *q = malloc(16);
  if (!p || !q)
    return 0;
  if (q - p == 64)
    reach_error();
  free(p);
  free(q);
  return 0;
}
