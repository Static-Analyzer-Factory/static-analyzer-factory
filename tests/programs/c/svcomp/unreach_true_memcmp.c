// unreach-call TRUE: memcmp is 0 on equal bytes and non-zero on different ones.
#include <string.h>
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

int main(void) {
  int a = __VERIFIER_nondet_int();
  int b = a;
  int c = a + 1;
  if (memcmp(&a, &b, sizeof a) != 0)
    reach_error();
  if (memcmp(&a, &c, sizeof a) == 0)
    reach_error();
  return 0;
}
