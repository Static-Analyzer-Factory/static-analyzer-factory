extern long long __VERIFIER_nondet_longlong(void);
int main(void){
  unsigned u = (unsigned)__VERIFIER_nondet_longlong();
  int x = (int)(u >> 1);
  int y = x + x;
  return y;
}
