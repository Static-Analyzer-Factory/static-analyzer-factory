// Large C++ program: plugin system with factory pattern and event bus
//
// This program is a stress test for SAF's pointer analysis. It combines
// multiple challenging patterns:
//   - Virtual dispatch (PluginBase hierarchy)
//   - Function pointer callbacks (EventBus listeners)
//   - Factory pattern with function pointer table (PluginRegistry)
//   - Dynamic object creation (new/delete)
//   - Array-based polymorphism (active_plugins array)
//
// The program simulates a plugin-based application server with:
//   - 4 concrete plugin types (Log, Auth, Cache, Metrics)
//   - A plugin registry with factory function pointers
//   - An event bus with subscriber callbacks
//   - An application class that ties it all together

#include <cstdio>
#include <cstdlib>
#include <cstring>

// --- Plugin Base ---
struct PluginBase {
    virtual const char *name() const = 0;
    virtual void init() = 0;
    virtual int execute(const char *input) = 0;
    virtual ~PluginBase() {}
};

// --- Concrete Plugins ---
struct LogPlugin : PluginBase {
    const char *name() const override { return "log"; }
    void init() override { printf("[LogPlugin] initialized\n"); }
    int execute(const char *input) override {
        printf("[LOG] %s\n", input);
        return 0;
    }
};

struct AuthPlugin : PluginBase {
    int authenticated;
    AuthPlugin() : authenticated(0) {}
    const char *name() const override { return "auth"; }
    void init() override {
        authenticated = 1;
        printf("[AuthPlugin] initialized\n");
    }
    int execute(const char *input) override {
        if (!authenticated) return -1;
        printf("[AUTH] checking: %s\n", input);
        return (strlen(input) > 3) ? 1 : 0;
    }
};

struct CachePlugin : PluginBase {
    char cache[256];
    CachePlugin() { cache[0] = '\0'; }
    const char *name() const override { return "cache"; }
    void init() override { printf("[CachePlugin] initialized\n"); }
    int execute(const char *input) override {
        if (strcmp(cache, input) == 0) {
            printf("[CACHE] hit: %s\n", input);
            return 1;
        }
        strncpy(cache, input, 255);
        cache[255] = '\0';
        printf("[CACHE] miss, stored: %s\n", input);
        return 0;
    }
};

struct MetricsPlugin : PluginBase {
    int call_count;
    MetricsPlugin() : call_count(0) {}
    const char *name() const override { return "metrics"; }
    void init() override {
        call_count = 0;
        printf("[MetricsPlugin] initialized\n");
    }
    int execute(const char *input) override {
        call_count++;
        printf("[METRICS] call #%d: %s\n", call_count, input);
        return call_count;
    }
};

// --- Plugin Factory ---
typedef PluginBase *(*PluginCreator)();

PluginBase *create_log() { return new LogPlugin(); }
PluginBase *create_auth() { return new AuthPlugin(); }
PluginBase *create_cache() { return new CachePlugin(); }
PluginBase *create_metrics() { return new MetricsPlugin(); }

struct RegistryEntry {
    const char *name;
    PluginCreator creator;
};

struct PluginRegistry {
    RegistryEntry entries[16];
    int count;

    PluginRegistry() : count(0) {}

    void register_plugin(const char *name, PluginCreator creator) {
        if (count < 16) {
            entries[count].name = name;
            entries[count].creator = creator;
            count++;
        }
    }

    PluginBase *create(const char *name) {
        for (int i = 0; i < count; i++) {
            if (strcmp(entries[i].name, name) == 0) {
                return entries[i].creator();  // indirect call via function pointer
            }
        }
        return nullptr;
    }
};

// --- Event System ---
typedef void (*EventListener)(const char *event, const char *data);

void on_plugin_init(const char *event, const char *data) {
    printf("[EVENT] %s: %s\n", event, data);
}

void on_plugin_execute(const char *event, const char *data) {
    printf("[EVENT] %s: processing %s\n", event, data);
}

void on_error(const char *event, const char *data) {
    printf("[EVENT] ERROR %s: %s\n", event, data);
}

struct EventBus {
    EventListener listeners[16];
    int count;

    EventBus() : count(0) {}

    void subscribe(EventListener listener) {
        if (count < 16) {
            listeners[count++] = listener;
        }
    }

    void emit(const char *event, const char *data) {
        for (int i = 0; i < count; i++) {
            listeners[i](event, data);  // indirect call
        }
    }
};

// --- Configuration ---
struct ConfigEntry {
    const char *key;
    const char *value;
};

struct AppConfig {
    ConfigEntry entries[32];
    int count;

    AppConfig() : count(0) {}

    void set(const char *key, const char *value) {
        // Update existing or add new
        for (int i = 0; i < count; i++) {
            if (strcmp(entries[i].key, key) == 0) {
                entries[i].value = value;
                return;
            }
        }
        if (count < 32) {
            entries[count].key = key;
            entries[count].value = value;
            count++;
        }
    }

    const char *get(const char *key) const {
        for (int i = 0; i < count; i++) {
            if (strcmp(entries[i].key, key) == 0) {
                return entries[i].value;
            }
        }
        return nullptr;
    }
};

// --- Health Check ---
struct HealthCheck {
    const char *component;
    int (*checker)(void);
};

int check_log() { return 1; }
int check_auth() { return 1; }
int check_cache() { return 1; }
int check_metrics() { return 1; }

struct HealthMonitor {
    HealthCheck checks[16];
    int count;

    HealthMonitor() : count(0) {}

    void add_check(const char *component, int (*checker)(void)) {
        if (count < 16) {
            checks[count].component = component;
            checks[count].checker = checker;
            count++;
        }
    }

    int run_all() {
        int all_ok = 1;
        for (int i = 0; i < count; i++) {
            int result = checks[i].checker();  // indirect call
            printf("[HEALTH] %s: %s\n", checks[i].component,
                   result ? "OK" : "FAIL");
            if (!result) all_ok = 0;
        }
        return all_ok;
    }
};

// --- Application ---
struct Application {
    PluginRegistry registry;
    EventBus events;
    AppConfig config;
    HealthMonitor health;
    PluginBase *active_plugins[16];
    int plugin_count;

    Application() : plugin_count(0) {}

    void setup() {
        // Register plugin factories
        registry.register_plugin("log", create_log);
        registry.register_plugin("auth", create_auth);
        registry.register_plugin("cache", create_cache);
        registry.register_plugin("metrics", create_metrics);

        // Subscribe event listeners
        events.subscribe(on_plugin_init);
        events.subscribe(on_plugin_execute);
        events.subscribe(on_error);

        // Set configuration
        config.set("app.name", "SAF Tutorial");
        config.set("app.version", "1.0");
        config.set("log.level", "info");
        config.set("cache.ttl", "300");

        // Register health checks
        health.add_check("log", check_log);
        health.add_check("auth", check_auth);
        health.add_check("cache", check_cache);
        health.add_check("metrics", check_metrics);
    }

    void activate(const char *name) {
        PluginBase *p = registry.create(name);  // factory + indirect call
        if (p) {
            p->init();                          // virtual dispatch
            active_plugins[plugin_count++] = p;
            events.emit("init", p->name());     // event callback + virtual dispatch
        } else {
            events.emit("error", "plugin not found");
        }
    }

    void run(const char *input) {
        events.emit("execute", input);  // event callbacks
        for (int i = 0; i < plugin_count; i++) {
            int result = active_plugins[i]->execute(input);  // virtual dispatch
            printf("  Plugin '%s' returned %d\n",
                   active_plugins[i]->name(), result);        // virtual dispatch
        }
    }

    int check_health() {
        return health.run_all();  // indirect calls through function pointers
    }

    void shutdown() {
        for (int i = 0; i < plugin_count; i++) {
            printf("Shutting down plugin: %s\n",
                   active_plugins[i]->name());    // virtual dispatch
            delete active_plugins[i];              // virtual destructor
        }
        plugin_count = 0;
    }
};

int main(void) {
    Application app;
    app.setup();

    printf("=== Activating plugins ===\n");
    app.activate("log");
    app.activate("auth");
    app.activate("cache");
    app.activate("metrics");

    printf("\n=== Configuration ===\n");
    printf("App: %s v%s\n",
           app.config.get("app.name"),
           app.config.get("app.version"));

    printf("\n=== Health Check ===\n");
    int healthy = app.check_health();
    printf("Overall health: %s\n", healthy ? "OK" : "DEGRADED");

    printf("\n=== Processing requests ===\n");
    app.run("test-request-1");
    app.run("test-request-2");
    app.run("test-request-1");  // cache hit on second call

    printf("\n=== Shutdown ===\n");
    app.shutdown();

    return 0;
}
