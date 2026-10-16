// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume_abort_if_not(int c) { if (!c) abort(); }
int N, x;
void *thr(void *a) { x = N - 1; return 0; }
int main(void) { pthread_t t; N = __VERIFIER_nondet_int(); assume_abort_if_not(N >= 0); N = __VERIFIER_nondet_int(); pthread_create(&t, 0, thr, 0); return 0; }
