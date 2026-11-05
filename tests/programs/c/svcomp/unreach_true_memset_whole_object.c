// unreach-call TRUE: memset to 0 over the whole block zeroes every field.
#include <stdlib.h>
#include <string.h>
extern void reach_error(void);
struct tbl { int n; int *buckets; int tail; };

int main(void) {
  struct tbl *t = malloc(sizeof *t);
  if (!t)
    return 0;
  memset(t, 0, sizeof *t);
  if (t->n != 0 || t->buckets != 0 || t->tail != 0)
    reach_error();
  free(t);
  return 0;
}
