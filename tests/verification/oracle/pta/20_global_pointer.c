// PURPOSE: Global variable pointer initialization
int g_val;
int *g_ptr = &g_val;

void test() {
    *g_ptr = 42;
}
