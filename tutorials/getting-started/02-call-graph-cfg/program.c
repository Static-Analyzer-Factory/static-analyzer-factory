// Call graph visualization: a multi-function C program with clear call relationships.
//
// Call structure:
//   main -> log_error, parse_request
//   parse_request -> validate_input, process_data, send_response
//   validate_input -> log_error
//   process_data -> (leaf: no callees besides stdlib)
//   send_response -> (leaf)
//   log_error -> (leaf)

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

void log_error(const char *msg) {
    fprintf(stderr, "ERROR: %s\n", msg);
}

int validate_input(const char *input) {
    if (!input || strlen(input) == 0) {
        log_error("empty input");
        return 0;
    }
    if (strlen(input) > 256) {
        log_error("input too long");
        return 0;
    }
    return 1;
}

char *process_data(const char *input) {
    char *result = malloc(512);
    if (!result) return NULL;
    snprintf(result, 512, "processed: %s", input);
    return result;
}

void send_response(const char *data) {
    printf("Response: %s\n", data);
}

int parse_request(const char *raw) {
    if (!validate_input(raw)) return -1;
    char *data = process_data(raw);
    if (!data) return -1;
    send_response(data);
    free(data);
    return 0;
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        log_error("usage: program <request>");
        return 1;
    }
    return parse_request(argv[1]);
}
