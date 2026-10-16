// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume_abort_if_not(int c) { if (!c) abort(); }
int n1, n2, tmp1, tmp2, i, i1, i2;
void *thread1(void *a) { tmp1 = n2; n1 = tmp1 + 1; i1 = i; i = i1 + 1; n1 = 0; return 0; }
void *thread2(void *a) { tmp2 = n1; n2 = tmp2 + 1; i2 = i; i = i2 + 1; n2 = 0; return 0; }
int main(void) { pthread_t t1, t2; pthread_create(&t1, 0, thread1, 0); pthread_create(&t2, 0, thread2, 0); return 0; }
