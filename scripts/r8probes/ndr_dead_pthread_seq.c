// pthread_create PRESENT but UNREACHABLE from main (dead scaffolding) + sequential main.
// The only shape in which R8 gets recall on a threading-linked task. Expect: true.
#include <pthread.h>
extern int __VERIFIER_nondet_int(void);
int g;
void *worker(void *a) { g++; return 0; }
void never_called(void) { pthread_t t; pthread_create(&t, 0, worker, 0); }
int main(void) { g = __VERIFIER_nondet_int(); return g; }
