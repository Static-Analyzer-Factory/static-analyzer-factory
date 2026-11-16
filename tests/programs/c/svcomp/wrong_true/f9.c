/* Does not terminate for x > 0 (x < INT_MAX never overflows here). */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  while (x > 0) {
    x = x + 1;
    x = x - 1;
  }
  return 0;
}
