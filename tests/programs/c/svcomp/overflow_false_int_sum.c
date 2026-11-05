// no-overflow: never TRUE. Two ints may overflow when added.
extern int __VERIFIER_nondet_int(void);

int main(void) {
  int a = __VERIFIER_nondet_int();
  int b = __VERIFIER_nondet_int();
  int s = a + b;
  return s > 0;
}
