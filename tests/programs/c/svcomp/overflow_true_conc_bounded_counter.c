// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
int cnt = 0;
void *thr(void *a) { cnt = cnt + 1; return 0; }
int main(void) { pthread_t t1, t2; pthread_create(&t1, 0, thr, 0); pthread_create(&t2, 0, thr, 0); cnt = cnt + 1; pthread_join(t1, 0); pthread_join(t2, 0); return cnt + 1; }
