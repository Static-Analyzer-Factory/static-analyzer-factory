/* Does not terminate for x != 0: ~x alternates x, -x - 1, x, ... */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  while (x != 0) {
    x = ~x;
  }
  return 0;
}
