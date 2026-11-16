#include <pthread.h>
struct P { pthread_mutex_t m; int v; } arr[2] = {{.v=1},{.v=2}};
int x;
void *t1(void*a){ pthread_mutex_lock(&arr[0].m); x++; pthread_mutex_unlock(&arr[0].m); return 0; }
void *t2(void*a){ pthread_mutex_lock(&arr[1].m); x++; pthread_mutex_unlock(&arr[1].m); return 0; }
int main(void){ pthread_t a,b; pthread_create(&a,0,t1,0); pthread_create(&b,0,t2,0); return 0; }
