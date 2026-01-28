// CWE-89: SQL Injection — Test Program for Static Analysis
// This is an INTENTIONALLY VULNERABLE program used as a test fixture
// for SAF's taint analysis. It demonstrates a SQL injection pattern
// where environment variable data flows into a SQL query.
//
// Expected SAF finding: getenv() -> sqlite3_exec() (taint flow)
#include <stdlib.h>
#include <string.h>

typedef struct sqlite3 sqlite3;
int sqlite3_exec(sqlite3 *, const char *sql, void *, void *, char **);

int main(void) {
    const char *user_input = getenv("QUERY");
    char *query_str = (char *)user_input;
    sqlite3 *db = NULL;
    sqlite3_exec(db, query_str, NULL, NULL, NULL);
    return 0;
}
