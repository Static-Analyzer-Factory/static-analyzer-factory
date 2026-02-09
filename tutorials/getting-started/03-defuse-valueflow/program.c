// Def-use chain exploration: a single allocation flows to multiple uses.
//
// The buffer allocated by malloc() is used by:
//   1. strcpy  (write data into buffer)
//   2. log_message (pass to a function)
//   3. printf  (print directly)
//   4. free    (deallocate)

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

void log_message(const char *msg) {
    printf("LOG: %s\n", msg);
}

int main(void) {
    // def: allocate a buffer
    char *buf = (char *)malloc(64);
    if (!buf) return 1;

    // use 1: copy data into buffer
    strcpy(buf, "hello world");

    // use 2: log the buffer contents
    log_message(buf);

    // use 3: print directly
    printf("Data: %s\n", buf);

    // use 4: free the buffer
    free(buf);

    return 0;
}
