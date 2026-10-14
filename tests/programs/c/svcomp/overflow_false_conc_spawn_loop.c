// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
int cnt = 0;
void *thr(void *a) { cnt = cnt + 1000000; return 0; }
int main(void) { pthread_t t; while (__VERIFIER_nondet_int()) pthread_create(&t, 0, thr, 0); return 0; }
