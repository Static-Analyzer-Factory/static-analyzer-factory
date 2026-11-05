// unreach-call TRUE: a pointer to a field, stepped back by the field's offset,
// is the record again (uthash's ELMT_FROM_HH): the store lands in `r->id`.
#include <stdlib.h>
extern void reach_error(void);
struct rec { int id; int cookie; };

int main(void) {
  struct rec *r = malloc(sizeof *r);
  if (!r)
    return 0;
  r->id = 1;
  long off = (char *)&r->cookie - (char *)r;
  struct rec *back = (struct rec *)((char *)&r->cookie - off);
  back->id = 7;
  if (r->id != 7)
    reach_error();
  free(r);
  return 0;
}
