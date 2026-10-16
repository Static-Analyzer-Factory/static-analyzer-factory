// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
unsigned int u, u2;
void *thr(void *a) { u2 = u + 2147483647u; u = u2 + 2147483647u; return 0; }
int main(void) { pthread_t t; pthread_create(&t, 0, thr, 0); int x = (int)u; int y = x - 1; return y; }
