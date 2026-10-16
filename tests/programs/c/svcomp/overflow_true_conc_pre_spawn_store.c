// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume_abort_if_not(int c) { if (!c) abort(); }
int x1;
void *thr(void *a) { while (x1 < 10000) { x1++; } return 0; }
int main(void) { pthread_t t; x1 = __VERIFIER_nondet_int(); assume_abort_if_not(x1 >= 0); pthread_create(&t, 0, thr, 0); pthread_join(t, 0); return 0; }
