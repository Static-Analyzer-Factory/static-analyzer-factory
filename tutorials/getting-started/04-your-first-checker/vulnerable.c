// Simple Memory Leak Example
//
// This program demonstrates a basic memory leak:
// memory is allocated but never freed on certain paths.

#include <stdlib.h>
#include <string.h>

char *create_greeting(const char *name) {
    // Allocate memory for the greeting
    char *greeting = (char *)malloc(256);
    if (!greeting) {
        return NULL;
    }

    // Build the greeting string
    strcpy(greeting, "Hello, ");
    strcat(greeting, name);
    strcat(greeting, "!");

    return greeting;  // Caller is responsible for freeing
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        return 1;
    }

    // Create a greeting - this allocates memory
    char *msg = create_greeting(argv[1]);

    if (msg) {
        // Use the greeting somehow...
        // But we forgot to free it!
        // free(msg);  // <-- MISSING: This causes a memory leak
    }

    return 0;
    // LEAK: msg is never freed before program exit
}
