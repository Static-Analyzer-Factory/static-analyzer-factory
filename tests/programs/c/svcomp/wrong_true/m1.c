#include <pthread.h>
void *f(void *a){ return 0; }
int main(void){ pthread_t t; char st;
  pthread_create(&t, 0, f, 0);
  pthread_join(t, (void**)&st);
  return 0; }
