// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume_abort_if_not(int c) { if (!c) abort(); }
int x1;
pthread_t t;
void *thr(void *a) { while (x1 < 10000) { x1++; } return 0; }
int main(void) { int k = 0; while (1) { x1 = __VERIFIER_nondet_int(); if (k == 0) { pthread_create(&t, 0, thr, 0); k = 1; } } return 0; }
