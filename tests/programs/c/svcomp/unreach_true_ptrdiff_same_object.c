// unreach-call TRUE: the distance between two fields of one heap object is a
// constant (uthash stores it as `hho` and steps back with it).
#include <stdlib.h>
extern void reach_error(void);
struct rec { int id; int cookie; struct rec *next; };

int main(void) {
  struct rec *r = malloc(sizeof *r);
  if (!r)
    return 0;
  long d = (char *)&r->next - (char *)r;
  if (d != (long)sizeof(int) * 2)
    reach_error();
  free(r);
  return 0;
}
