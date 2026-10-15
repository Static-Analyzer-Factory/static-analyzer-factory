// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume(int c) { if (!c) abort(); }
int plus(int a, int b) { assume(b <= 0 || a <= INT_MAX - b); assume(b >= 0 || a >= INT_MIN - b); return a + b; }
int r;
void *thr(void *x) { int s = plus(__VERIFIER_nondet_int(), __VERIFIER_nondet_int()); (void)s; return 0; }
int main(void) { pthread_t t; pthread_create(&t, 0, thr, 0); return 0; }
