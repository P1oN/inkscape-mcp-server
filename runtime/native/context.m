/* GTK 3 module. Uses only public GTK/GIO APIs, no Inkscape C++ ABI.
 * The D-Bus callback and GAction activation run in the same GUI main context.
 * UUIDs belong to live GObjects; replacing a document action group changes identity.
 */
#include <Cocoa/Cocoa.h>
#include <gio/gio.h>
#include <string.h>
#include <stdlib.h>

typedef void GtkApplication;
typedef void GtkWindow;
extern GtkWindow *gtk_application_get_active_window(GtkApplication *app);
extern GList *gtk_application_get_windows(GtkApplication *app);
extern GActionGroup *gtk_widget_get_action_group(void *widget, const char *name);
extern const char *gtk_window_get_title(GtkWindow *window);
extern void gtk_window_present(GtkWindow *window);
extern void *gdk_display_get_default(void);
extern void *gdk_display_get_primary_monitor(void *display);

static const char *XML =
"<node><interface name='org.inkscape.MCP.Context1'>"
"<method name='GetContext'><arg type='s' direction='out'/>"
"<arg type='s' direction='out'/><arg type='s' direction='out'/></method>"
"<method name='ListDocuments'><arg type='a(sss)' direction='out'/></method>"
"<method name='SelectDocument'><arg type='s' direction='in'/>"
"<arg type='s' direction='in'/></method>"
"<method name='Activate'><arg type='s' direction='in'/><arg type='s' direction='in'/>"
"<arg type='s' direction='in'/><arg type='av' direction='in'/></method>"
"</interface></node>";

static const char *identity(GObject *object) {
    const char *id = g_object_get_data(object, "inkscape-mcp-context-id");
    if (!id) {
        id = g_uuid_string_random();
        g_object_set_data_full(object, "inkscape-mcp-context-id", (void *)id, g_free);
    }
    return id;
}

static GActionGroup *document_group(GtkWindow *window) {
    return window ? gtk_widget_get_action_group(window, "doc") : NULL;
}

static GVariant *context(GtkWindow *window) {
    GActionGroup *doc = document_group(window);
    const char *title = window ? gtk_window_get_title(window) : NULL;
    return g_variant_new("(sss)", doc ? identity(G_OBJECT(window)) : "",
                         doc ? identity(G_OBJECT(doc)) : "", title ? title : "");
}

static void refuse(GDBusMethodInvocation *call, const char *message) {
    g_dbus_method_invocation_return_dbus_error(call, "org.inkscape.MCP.ContextChanged", message);
}

/* Fixed internal adapter surface. Never export arbitrary GUI actions. */
static gboolean allowed(const char *action) {
    const char *names[] = {
        "export-id", "export-id-only", "export-text-to-path", "export-plain-svg",
        "export-filename", "export-type", "export-area", "export-area-page", "export-dpi",
        "export-do", "select-list", "query-x", "object-set-property",
        "org.inkscape-mcp.insert.noprefs", "org.inkscape-mcp.edit.noprefs", "org.inkscape-mcp.live.noprefs", NULL
    };
    for (int i = 0; names[i]; i++) if (!strcmp(names[i], action)) return TRUE;
    return FALSE;
}

static void method(GDBusConnection *bus, const char *sender, const char *path,
                   const char *iface, const char *name, GVariant *params,
                   GDBusMethodInvocation *call, void *data) {
    (void)bus; (void)sender; (void)path; (void)iface; (void)data;
    GApplication *app = g_application_get_default();
    if (!app) { refuse(call, "No application"); return; }
    if (!strcmp(name, "GetContext")) {
        g_dbus_method_invocation_return_value(call, context(gtk_application_get_active_window(app)));
        return;
    }
    if (!strcmp(name, "ListDocuments")) {
        GVariantBuilder rows;
        g_variant_builder_init(&rows, G_VARIANT_TYPE("a(sss)"));
        for (GList *item = gtk_application_get_windows(app); item; item = item->next) {
            if (document_group(item->data)) g_variant_builder_add_value(&rows, context(item->data));
        }
        g_dbus_method_invocation_return_value(call, g_variant_new("(a(sss))", &rows));
        return;
    }
    const char *window_id, *document_id;
    if (!strcmp(name, "SelectDocument")) {
        g_variant_get(params, "(&s&s)", &window_id, &document_id);
        for (GList *item = gtk_application_get_windows(app); item; item = item->next) {
            GActionGroup *doc = document_group(item->data);
            if (doc && !strcmp(identity(item->data), window_id) &&
                !strcmp(identity(G_OBJECT(doc)), document_id)) {
                [NSApp activateIgnoringOtherApps:YES];
                gtk_window_present(item->data);
                g_dbus_method_invocation_return_value(call, NULL);
                return;
            }
        }
        refuse(call, "Selected document was closed or replaced");
        return;
    }
    const char *action;
    GVariant *arguments;
    g_variant_get(params, "(&s&s&s@av)", &window_id, &document_id, &action, &arguments);
    GtkWindow *window = gtk_application_get_active_window(app);
    GActionGroup *doc = document_group(window);
    if (!doc || strcmp(identity(G_OBJECT(window)), window_id) ||
        strcmp(identity(G_OBJECT(doc)), document_id)) {
        g_variant_unref(arguments);
        refuse(call, "Active document changed; select the intended document again");
        return;
    }
    /* The noprefs effect action exists only on the app group in Inkscape 1.4.3.
     * All actions resolve the active document synchronously here, after the guard,
     * before control returns to GTK. The effect captures that document for its run. */
    GActionGroup *group = G_ACTION_GROUP(app);
    if (!allowed(action) || !g_action_group_has_action(group, action)) {
        g_variant_unref(arguments);
        refuse(call, "Unsupported context action");
        return;
    }
    const GVariantType *type = g_action_group_get_action_parameter_type(group, action);
    gsize count = g_variant_n_children(arguments);
    GVariant *child = count == 1 ? g_variant_get_child_value(arguments, 0) : NULL;
    GVariant *value = child ? g_variant_get_variant(child) : NULL;
    if (child) g_variant_unref(child);
    if (!allowed(action) || !g_action_group_has_action(group, action) ||
        !g_action_group_get_action_enabled(group, action) || count > 1 ||
        (type ? (!value || !g_variant_is_of_type(value, type)) : value != NULL)) {
        if (value) g_variant_unref(value);
        g_variant_unref(arguments);
        refuse(call, "Unsupported context action or parameter");
        return;
    }
    g_action_group_activate_action(group, action, value);
    if (value) g_variant_unref(value);
    g_variant_unref(arguments);
    g_dbus_method_invocation_return_value(call, NULL);
}

static const GDBusInterfaceVTable VTABLE = { method, NULL, NULL, {0} };

static gboolean install(void *data) {
    (void)data;
    GApplication *app = g_application_get_default();
    if (!app || !g_application_get_is_registered(app)) return G_SOURCE_CONTINUE;
    GDBusConnection *bus = g_application_get_dbus_connection(app);
    if (!bus) return G_SOURCE_REMOVE;
    GError *error = NULL;
    GDBusNodeInfo *info = g_dbus_node_info_new_for_xml(XML, &error);
    if (info) {
        g_dbus_connection_register_object(bus, "/org/inkscape/Inkscape/MCPContext",
                                         info->interfaces[0], &VTABLE, NULL, NULL, &error);
        g_dbus_node_info_unref(info);
    }
    if (error) { g_warning("MCP context bridge: %s", error->message); g_error_free(error); }
    return G_SOURCE_REMOVE;
}

void gtk_module_init(int *argc, char ***argv) {
    (void)argc; (void)argv;
    if (g_getenv("INKSCAPE_MCP_MANAGED_DIR")) {
        void *display = gdk_display_get_default();
        if (!display || !gdk_display_get_primary_monitor(display)) {
            /* Inkscape 1.4.3 dereferences the primary monitor during window creation.
             * Stop before any drawing is opened, instead of entering its crash dialog. */
            g_printerr("MCP startup refused: no primary monitor; unlock the Mac and retry.\n");
            _Exit(EXIT_FAILURE);
        }
        [NSApp activateIgnoringOtherApps:YES];
        g_timeout_add(50, install, NULL);
    }
}
