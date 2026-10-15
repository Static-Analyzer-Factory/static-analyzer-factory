// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
#include <limits.h>
extern int __VERIFIER_nondet_int(void);
extern void abort(void);
void assume(int c) { if (!c) abort(); }
int g;
int bump(int a) { return g + a; }
void *w(void *x) { g = INT_MAX; return 0; }
void *thr(void *x) { int s = bump(1); return (void *)(long)s; }
int main(void) { pthread_t t, u; pthread_create(&u, 0, w, 0); pthread_create(&t, 0, thr, 0); return 0; }
