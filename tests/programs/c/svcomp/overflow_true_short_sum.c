// no-overflow TRUE: ten shorts summed in an int, then subtracted again, never
// leave int (the solver alone times out on this adder chain).
extern short __VERIFIER_nondet_short(void);

int main(void) {
  short v[10];
  int sum = 0;
  for (int i = 0; i < 10; i++) {
    v[i] = __VERIFIER_nondet_short();
    sum += v[i];
  }
  for (int i = 0; i < 10; i++)
    sum -= v[i];
  return sum;
}
