// unreach-call: never TRUE. A nondet key equals 5 on some run.
#include <string.h>
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

int main(void) {
  int a = __VERIFIER_nondet_int();
  int b = 5;
  if (memcmp(&a, &b, sizeof a) == 0)
    reach_error();
  return 0;
}
