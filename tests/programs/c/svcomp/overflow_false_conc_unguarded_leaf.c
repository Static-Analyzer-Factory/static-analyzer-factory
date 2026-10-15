// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume(int c) { if (!c) abort(); }
int plus(int a, int b) { return a + b; }
void *thr(void *x) { int s = plus(__VERIFIER_nondet_int(), __VERIFIER_nondet_int()); return (void *)(long)s; }
int main(void) { pthread_t t; pthread_create(&t, 0, thr, 0); return 0; }
