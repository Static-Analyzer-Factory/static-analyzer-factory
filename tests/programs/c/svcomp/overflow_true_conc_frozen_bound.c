// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume_abort_if_not(int c) { if (!c) abort(); }
int N, x;
void *thr(void *a) { for (int i = 0; i < N - 1; i++) { x = i + 1; } return 0; }
int main(void) { pthread_t t; int n = __VERIFIER_nondet_int(); assume_abort_if_not(n > 0); N = n; pthread_create(&t, 0, thr, 0); return 0; }
