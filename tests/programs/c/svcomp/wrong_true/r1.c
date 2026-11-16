#include <pthread.h>
int x; int *gp;
void *f(void *arg){ gp = (int*)arg; *gp = 1; return 0; }
int main(void){ pthread_t t; pthread_create(&t,0,f,&x); x = 2; pthread_join(t,0); return 0; }
