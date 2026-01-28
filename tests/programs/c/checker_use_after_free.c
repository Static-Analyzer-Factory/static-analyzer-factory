#include <stdlib.h>
#include <string.h>

void process() {
    char *buffer = (char *)malloc(64);
    strcpy(buffer, "Hello, allocated memory!");

    // Free the memory
    free(buffer);

    // BUG: use-after-free — reading freed memory
    char c = buffer[0];

    // BUG: use-after-free — writing to freed memory
    buffer[0] = 'X';
}

int main() {
    process();
    return 0;
}
