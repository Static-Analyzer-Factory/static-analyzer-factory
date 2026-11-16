/* Large x overflows x * 2 (excluded), small x > 0 loops forever. */
extern int __VERIFIER_nondet_int(void);
int main(void) {
  int x = __VERIFIER_nondet_int();
  int y = 0;
  while (x > 0) {
    y = x * 2;
    if (y < 0)
      break;
  }
  return y;
}
