// CONTROL: genuinely sequential, no threads. R8 SHOULD emit true (no data race).
// Proves the mechanism works when a task is actually sequential.
extern int __VERIFIER_nondet_int(void);
int g;
int main(void) {
  int n = __VERIFIER_nondet_int();
  for (int i = 0; i < 3; i++) g += (n & 1);
  return g;
}
