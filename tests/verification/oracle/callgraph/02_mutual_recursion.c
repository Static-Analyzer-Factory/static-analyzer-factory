// PURPOSE: Mutually recursive functions
void ping(int n);
void pong(int n);

void ping(int n) { if (n > 0) pong(n - 1); }
void pong(int n) { if (n > 0) ping(n - 1); }

void test() { ping(10); }
