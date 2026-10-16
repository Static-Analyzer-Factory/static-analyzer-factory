// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <stdlib.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume_abort_if_not(int c) { if (!c) abort(); }
int front, back, n;
void *thr(void *a) { while (1) { assume_abort_if_not(back > front && front < n); front++; } return 0; }
int main(void) { pthread_t t; n = __VERIFIER_nondet_int(); back = __VERIFIER_nondet_int(); pthread_create(&t, 0, thr, 0); return 0; }
