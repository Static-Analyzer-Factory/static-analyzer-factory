// unreach-call: never TRUE. Only the first field is cleared; `tail` holds
// whatever malloc left there.
#include <stdlib.h>
#include <string.h>
extern void reach_error(void);
struct tbl { int n; int tail; };

int main(void) {
  struct tbl *t = malloc(sizeof *t);
  if (!t)
    return 0;
  memset(t, 0, sizeof(int));
  if (t->tail != 0)
    reach_error();
  free(t);
  return 0;
}
