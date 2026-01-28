#include <stdlib.h>
// getenv → system (direct, single function)
int main() {
    char *path = getenv("PATH");
    system(path);
    return 0;
}
