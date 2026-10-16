// Concurrent no-overflow prover fixture (plan 222 §2z).
#include <pthread.h>
unsigned int u;
void *thr(void *a) { u = u + 2147483647u; u = u + 2147483647u; return 0; }
int main(void) { pthread_t t; pthread_create(&t, 0, thr, 0); int x = (int)u; int y = x - 1; return y; }
