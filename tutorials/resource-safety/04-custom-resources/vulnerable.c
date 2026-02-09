/**
 * Database connection handler with resource management bugs.
 *
 * Demonstrates custom resource tracking for domain-specific resources
 * that SAF doesn't have built-in specifications for.
 *
 * Bugs:
 * 1. Connection leak: db_connect without db_disconnect on error path
 * 2. Use-after-disconnect: query after db_disconnect
 * 3. Double-disconnect: calling db_disconnect twice
 *
 * CWE-772: Missing Release of Resource after Effective Lifetime
 * CWE-416: Use After Free (applied to non-memory resources)
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Opaque database connection handle */
typedef struct db_connection {
    int fd;
    char *host;
    int connected;
} db_connection_t;

/* Simulated database API */
db_connection_t *db_connect(const char *host, int port) {
    db_connection_t *conn = (db_connection_t *)malloc(sizeof(db_connection_t));
    if (!conn) return NULL;
    conn->host = strdup(host);
    conn->fd = port;  /* Simplified: use port as fake fd */
    conn->connected = 1;
    return conn;
}

int db_query(db_connection_t *conn, const char *sql) {
    if (!conn || !conn->connected) return -1;
    /* Simulate query execution */
    printf("Executing: %s\n", sql);
    return 0;
}

int db_disconnect(db_connection_t *conn) {
    if (!conn) return -1;
    if (conn->host) free(conn->host);
    conn->connected = 0;
    free(conn);
    return 0;
}

/**
 * BUG 1: Connection leak on error path.
 *
 * Opens a connection but doesn't close it if the first query fails.
 */
int process_data_leak(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    if (!conn) {
        return -1;
    }

    /* First query */
    if (db_query(conn, "SELECT * FROM users") != 0) {
        /* BUG: Connection leaked here! Should call db_disconnect(conn) */
        return -1;
    }

    /* Second query */
    if (db_query(conn, "UPDATE users SET active=1") != 0) {
        db_disconnect(conn);
        return -1;
    }

    db_disconnect(conn);
    return 0;
}

/**
 * BUG 2: Use-after-disconnect.
 *
 * Calls db_query after the connection has been closed.
 */
int process_data_use_after_close(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    if (!conn) {
        return -1;
    }

    db_query(conn, "SELECT * FROM users");
    db_disconnect(conn);

    /* BUG: Using connection after disconnect */
    db_query(conn, "SELECT * FROM orders");

    return 0;
}

/**
 * BUG 3: Double-disconnect.
 *
 * Calls db_disconnect twice on the same connection.
 */
int process_data_double_close(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    if (!conn) {
        return -1;
    }

    db_query(conn, "SELECT * FROM users");
    db_disconnect(conn);

    /* BUG: Double-disconnect (double-free of underlying memory) */
    db_disconnect(conn);

    return 0;
}

/**
 * CORRECT: Proper connection lifecycle management.
 */
int process_data_correct(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    if (!conn) {
        return -1;
    }

    /* First query */
    if (db_query(conn, "SELECT * FROM users") != 0) {
        db_disconnect(conn);  /* Properly close on error */
        return -1;
    }

    /* Second query */
    if (db_query(conn, "UPDATE users SET active=1") != 0) {
        db_disconnect(conn);  /* Properly close on error */
        return -1;
    }

    db_disconnect(conn);  /* Properly close on success */
    return 0;
}

int main(void) {
    /* These would demonstrate the bugs at runtime */
    process_data_leak("localhost");
    process_data_correct("localhost");
    return 0;
}
