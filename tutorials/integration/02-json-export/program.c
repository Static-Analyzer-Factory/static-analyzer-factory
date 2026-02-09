// Multi-function program for graph export demonstration
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char *read_input(void) {
    char *buf = (char *)malloc(256);
    if (!buf) return NULL;
    printf("Enter data: ");
    if (!fgets(buf, 256, stdin)) {
        free(buf);
        return NULL;
    }
    return buf;
}

int validate(const char *data) {
    if (!data) return 0;
    return strlen(data) > 0 && strlen(data) < 200;
}

void process(const char *data) {
    printf("Processing: %s", data);
}

int main(void) {
    char *input = read_input();
    if (!input) return 1;
    if (validate(input)) {
        process(input);
    } else {
        printf("Invalid input\n");
    }
    free(input);
    return 0;
}
