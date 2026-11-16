extern int __VERIFIER_nondet_int(void);
int main(void){ int x = __VERIFIER_nondet_int();
  if (x < -100 || x > 100) return 0;
  while ((unsigned)(x + 5) > 10u && x < 100) { if (x == -6) x = 50; else x = x - 1; }
  return 0; }
