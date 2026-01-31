#include "aliascheck.h"

char * g(char * a) {
    *a = 0;
    return a;
}

char * f(char * a) {
   char * b ;

   b = a;
    *(b) = 0;
   return g(b);
}

void swap(char **a, char **b) {
   char * c;
   c = *a;
   *a = *b;
   *b = c;
}

char * c;
int main (){
    char b[20];
    char a[20];
    char * p1, *p2;

    p1 = a;
    p2 = b;

    swap (&p1, &p2);
    c = b;
    c = f(c);
    g(c);

    MAYALIAS(c, b);
    MAYALIAS(p1, a);
    MAYALIAS(p1, b);
    MAYALIAS(p2, a);
    MAYALIAS(p2, b);
    NOALIAS(a, b);
}
