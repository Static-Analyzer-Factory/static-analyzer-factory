#include <stdlib.h>
#include <string.h>
// getenv → sanitize (overwrite) → system (safe)
int main() {
    char *input = getenv("PATH");
    char safe[256];
    strcpy(safe, "/usr/bin/ls");  // overwrite with safe value
    system(safe);
    return 0;
}
