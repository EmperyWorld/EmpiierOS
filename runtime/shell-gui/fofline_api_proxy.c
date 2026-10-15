#define _POSIX_C_SOURCE 200809L

#include "fofline_api_proxy.h"

#include <glib.h>
#include <string.h>
#include <syslog.h>

#define API_PREFIX "/api/v1/"
#define API_HOST "https://app.eliteew.ru"
#define MAX_API_PATH 512U
#define MAX_API_QUERY 2048U
#define MAX_API_BODY (2U * 1024U * 1024U)
#define MAX_API_RESPONSE (8U * 1024U * 1024U)
#define MAX_CONTENT_RESPONSE (64U * 1024U * 1024U)
#define API_TIMEOUT_SEC 20U
#define CONTENT_TIMEOUT_SEC 120U

typedef struct {
    WebKitURISchemeRequest *request;
    SoupMessage *message;
    char *path;
    gboolean hangup;
    gboolean content_download;
    gboolean media_range;
    char *allow_origin;
} ApiProxyJob;

static EmpiierNativeBridge *proxy_bridge;

void empiier_fofline_api_proxy_bind(EmpiierNativeBridge *bridge) {
    proxy_bridge = bridge;
}

gboolean empiier_fofline_video_call_end_path(const char *path) {
    static const char prefix[] = "/api/v1/video-calls/";
    const char *id;
    size_t n;

    if (path == NULL || !g_str_has_prefix(path, prefix)) {
        return FALSE;
    }
    id = path + sizeof(prefix) - 1U;
    n = 0U;
    while (id[n] != '\0' && id[n] != '/') {
        char item = id[n];
        if (!g_ascii_isxdigit((guchar)item) && item != '-') {
            return FALSE;
        }
        n += 1U;
    }
    return n == 36U && g_strcmp0(id + n, "/end") == 0;
}

static gboolean path_is_content_download(const char *path) {
    if (path == NULL || !g_str_has_prefix(path, API_PREFIX "content/")) {
        return FALSE;
    }
    return g_str_has_suffix(path, "/download")
        || g_str_has_suffix(path, "/download-by-ticket")
        || g_str_has_suffix(path, "/batch-download-by-ticket");
}

static SoupSession *api_session(gboolean content_download) {
    static SoupSession *session = NULL;
    static SoupSession *content_session = NULL;

    if (content_download) {
        if (content_session == NULL) {
            content_session = soup_session_new();
            soup_session_set_timeout(content_session, CONTENT_TIMEOUT_SEC);
            soup_session_set_user_agent(content_session, "EmpiierOS-FoFline-API/1");
        }
        return content_session;
    }
    if (session == NULL) {
        session = soup_session_new();
        soup_session_set_timeout(session, API_TIMEOUT_SEC);
        soup_session_set_user_agent(session, "EmpiierOS-FoFline-API/1");
    }
    return session;
}

static gboolean path_chars_ok(const char *path) {
    if (path == NULL) {
        return FALSE;
    }
    for (const unsigned char *cursor = (const unsigned char *)path; *cursor != '\0'; cursor += 1U) {
        unsigned char item = *cursor;
        if ((item >= 'A' && item <= 'Z')
            || (item >= 'a' && item <= 'z')
            || (item >= '0' && item <= '9')
            || item == '/'
            || item == '-'
            || item == '_'
            || item == '.'
            || item == '~') {
            continue;
        }
        return FALSE;
    }
    return TRUE;
}

static gboolean query_chars_ok(const char *query) {
    if (query == NULL || query[0] == '\0') {
        return TRUE;
    }
    for (const unsigned char *cursor = (const unsigned char *)query; *cursor != '\0'; cursor += 1U) {
        unsigned char item = *cursor;
        if ((item >= 'A' && item <= 'Z')
            || (item >= 'a' && item <= 'z')
            || (item >= '0' && item <= '9')
            || item == '='
            || item == '&'
            || item == '%'
            || item == '_'
            || item == '-'
            || item == '.'
            || item == '+'
            || item == ','
            || item == ':') {
            continue;
        }
        return FALSE;
    }
    return TRUE;
}

gboolean empiier_fofline_api_path(const char *path) {
    return path != NULL
        && g_str_has_prefix(path, API_PREFIX)
        && strlen(path) <= MAX_API_PATH
        && strstr(path, "..") == NULL
        && strchr(path, '\\') == NULL
        && path_chars_ok(path);
}

static gboolean method_ok(const char *method) {
    return g_strcmp0(method, "GET") == 0
        || g_strcmp0(method, "HEAD") == 0
        || g_strcmp0(method, "POST") == 0
        || g_strcmp0(method, "PUT") == 0
        || g_strcmp0(method, "PATCH") == 0
        || g_strcmp0(method, "DELETE") == 0
        || g_strcmp0(method, "OPTIONS") == 0;
}

static gboolean header_forward(const char *name) {
    static const char *const allow[] = {
        "Authorization",
        "Accept",
        "Accept-Language",
        "Content-Type",
        "X-SSO-Replace-Confirm",
        "If-None-Match",
        "If-Modified-Since",
        "Cache-Control",
        "Pragma",
        "Prefer",
        "X-Requested-With",
        NULL
    };
    for (guint index = 0U; allow[index] != NULL; index += 1U) {
        if (g_ascii_strcasecmp(name, allow[index]) == 0) {
            return TRUE;
        }
    }
    return FALSE;
}

static void copy_request_header(
    const char *name,
    const char *value,
    gpointer user_data
) {
    SoupMessageHeaders *headers = user_data;
    if (name == NULL || value == NULL || !header_forward(name)) {
        return;
    }
    soup_message_headers_replace(headers, name, value);
}

static void finish_proxy_error(WebKitURISchemeRequest *request, const char *message) {
    GError *error = g_error_new_literal(G_IO_ERROR, G_IO_ERROR_FAILED, message);
    syslog(LOG_ERR, "fofline api proxy denied detail=%s", message);
    webkit_uri_scheme_request_finish_error(request, error);
    g_error_free(error);
}

static GBytes *read_body_bounded(GInputStream *stream, GError **error) {
    if (stream == NULL) {
        return NULL;
    }
    GByteArray *bytes = g_byte_array_new();
    guchar buffer[4096];
    while (TRUE) {
        gssize got = g_input_stream_read(stream, buffer, sizeof buffer, NULL, error);
        if (got < 0) {
            g_byte_array_unref(bytes);
            return NULL;
        }
        if (got == 0) {
            break;
        }
        if ((guint64)bytes->len + (guint64)got > MAX_API_BODY) {
            g_byte_array_unref(bytes);
            g_set_error_literal(error, G_IO_ERROR, G_IO_ERROR_FAILED, "api_body_too_large");
            return NULL;
        }
        g_byte_array_append(bytes, buffer, (guint)got);
    }
    return g_byte_array_free_to_bytes(bytes);
}

static const char *sniff_audio_type(const guint8 *data, gsize length) {
    if (data == NULL || length < 12U) {
        return NULL;
    }
    if (data[0] == 'I' && data[1] == 'D' && data[2] == '3') {
        return "audio/mpeg";
    }
    if (data[0] == 0xFFU && (data[1] & 0xE0U) == 0xE0U) {
        return "audio/mpeg";
    }
    if (data[0] == 'f' && data[1] == 'L' && data[2] == 'a' && data[3] == 'C') {
        return "audio/flac";
    }
    if (data[0] == 'O' && data[1] == 'g' && data[2] == 'g' && data[3] == 'S') {
        return "audio/ogg";
    }
    if (data[0] == 'R'
            && data[1] == 'I'
            && data[2] == 'F'
            && data[3] == 'F'
            && data[8] == 'W'
            && data[9] == 'A'
            && data[10] == 'V'
            && data[11] == 'E') {
        return "audio/wav";
    }
    if (data[4] == 'f' && data[5] == 't' && data[6] == 'y' && data[7] == 'p') {
        return "audio/mp4";
    }
    return NULL;
}

static void copy_json_reason(const char *body, gsize length, char *out, gsize out_len) {
    static const char key[] = "\"reason\":\"";
    const char *found = NULL;
    const char *end = body + length;
    if (body == NULL || out == NULL || out_len == 0U) {
        return;
    }
    out[0] = '\0';
    for (const char *cursor = body; cursor + sizeof(key) - 1U <= end; cursor += 1) {
        if (memcmp(cursor, key, sizeof(key) - 1U) == 0) {
            found = cursor + sizeof(key) - 1U;
            break;
        }
    }
    if (found == NULL) {
        return;
    }
    gsize index = 0U;
    while (index + 1U < out_len && found + index < end && found[index] != '"' && found[index] != '\\') {
        char item = found[index];
        if ((item >= 'a' && item <= 'z')
            || (item >= 'A' && item <= 'Z')
            || (item >= '0' && item <= '9')
            || item == '_'
            || item == '-') {
            out[index] = item;
            index += 1U;
            continue;
        }
        break;
    }
    out[index] = '\0';
}

static void finish_proxy_job(ApiProxyJob *job) {
    if (job == NULL) {
        return;
    }
    if (job->request != NULL) {
        g_object_unref(job->request);
    }
    if (job->message != NULL) {
        g_object_unref(job->message);
    }
    g_free(job->path);
    g_free(job->allow_origin);
    g_free(job);
}

static void proxy_finished(
    GObject *source,
    GAsyncResult *result,
    gpointer user_data
) {
    ApiProxyJob *job = user_data;
    GError *error = NULL;
    GBytes *body = soup_session_send_and_read_finish(SOUP_SESSION(source), result, &error);
    if (body == NULL) {
        syslog(
            LOG_ERR,
            "fofline api proxy transport failed detail=%s",
            error != NULL ? error->message : "unknown"
        );
        g_clear_error(&error);
        finish_proxy_error(job->request, "api_proxy_transport");
        finish_proxy_job(job);
        return;
    }
    gsize length = 0U;
    gconstpointer data = g_bytes_get_data(body, &length);
    gsize max_bytes = job->content_download ? MAX_CONTENT_RESPONSE : MAX_API_RESPONSE;
    if (length > max_bytes) {
        g_bytes_unref(body);
        finish_proxy_error(job->request, "api_proxy_response_too_large");
        finish_proxy_job(job);
        return;
    }
    guint status = (guint)soup_message_get_status(job->message);
    const char *reason = soup_message_get_reason_phrase(job->message);
    SoupMessageHeaders *response_headers = soup_message_get_response_headers(job->message);
    const char *content_type = response_headers != NULL
        ? soup_message_headers_get_one(response_headers, "Content-Type")
        : NULL;
    const char *serve_type = content_type != NULL ? content_type : "application/octet-stream";
    const char *sniffed = NULL;
    /* XHR blob fetch must stay octet-stream: an audio MIME on
     * fofline-spa: is handed to GStreamer and the XHR is cancelled.
     * The MusicPlayer stream path is a media element that sent Range;
     * that client needs a sniffed audio MIME or canplay never fires. */
    if (job->content_download && status == 206U) {
        g_bytes_unref(body);
        syslog(
            LOG_ERR,
            "fofline api proxy content range rejected status=206 bytes=%zu path=%s",
            length,
            job->path != NULL ? job->path : "-"
        );
        finish_proxy_error(job->request, "api_proxy_partial_content");
        finish_proxy_job(job);
        return;
    }
    if (job->content_download && status < 400U) {
        sniffed = sniff_audio_type(data, length);
        if (job->media_range && sniffed != NULL) {
            serve_type = sniffed;
        } else {
            serve_type = "application/octet-stream";
        }
    }
    guint8 *copy = g_malloc(length > 0U ? length : 1U);
    if (length > 0U && data != NULL) {
        memcpy(copy, data, length);
    }
    g_bytes_unref(body);
    GInputStream *stream = g_memory_input_stream_new_from_data(copy, (gssize)length, g_free);
    WebKitURISchemeResponse *response = webkit_uri_scheme_response_new(stream, (gint64)length);
    SoupMessageHeaders *out_headers = soup_message_headers_new(SOUP_MESSAGE_HEADERS_RESPONSE);
    char length_text[32];
    webkit_uri_scheme_response_set_status(
        response,
        status >= 100U ? status : 502U,
        reason
    );
    soup_message_headers_append(
        out_headers,
        "Access-Control-Allow-Origin",
        job->allow_origin != NULL ? job->allow_origin : "fofline-spa://localhost"
    );
    soup_message_headers_append(out_headers, "Access-Control-Allow-Credentials", "true");
    soup_message_headers_append(out_headers, "Vary", "Origin");
    soup_message_headers_append(
        out_headers,
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"
    );
    soup_message_headers_append(
        out_headers,
        "Access-Control-Allow-Headers",
        "Authorization, Content-Type, Accept, Range, X-SSO-Replace-Confirm, X-Requested-With"
    );
    soup_message_headers_append(
        out_headers,
        "Access-Control-Expose-Headers",
        "Content-Type, Content-Length, Content-Disposition"
    );
    soup_message_headers_append(out_headers, "X-Content-Type-Options", "nosniff");
    if (g_snprintf(length_text, sizeof(length_text), "%zu", length) > 0) {
        soup_message_headers_append(out_headers, "Content-Length", length_text);
    }
    webkit_uri_scheme_response_set_http_headers(response, out_headers);
    webkit_uri_scheme_response_set_content_type(response, serve_type);
    webkit_uri_scheme_request_finish_with_response(job->request, response);
    char reason_code[48];
    reason_code[0] = '\0';
    if (status >= 400U && length > 0U && length <= 256U) {
        copy_json_reason((const char *)copy, length, reason_code, sizeof reason_code);
    }
    if (reason_code[0] != '\0') {
        syslog(
            LOG_INFO,
            "fofline api proxy status=%u bytes=%zu path=%s reason=%s",
            status,
            length,
            job->path != NULL ? job->path : "-",
            reason_code
        );
    } else {
        syslog(
            LOG_INFO,
            "fofline api proxy status=%u bytes=%zu%s",
            status,
            length,
            job->content_download && status < 400U
                ? (job->media_range && sniffed != NULL ? " content=audio" : " content=octet-stream")
                : ""
        );
    }
    if (job->hangup
            && (status < 400U || status == 409U)
            && proxy_bridge != NULL
            && (proxy_bridge->meet_visible || proxy_bridge->meet_has_bounds)) {
        syslog(LOG_INFO, "fofline meet stage request kind=close code=video_call_end");
        empiier_native_bridge_hide_meet(proxy_bridge);
    }
    g_object_unref(response);
    g_object_unref(stream);
    finish_proxy_job(job);
}

void empiier_fofline_api_proxy(
    WebKitURISchemeRequest *request,
    const char *path,
    const char *query
) {
    if (request == NULL || !empiier_fofline_api_path(path)) {
        if (request != NULL) {
            finish_proxy_error(request, "api_path_denied");
        }
        return;
    }
    if (query != NULL && (strlen(query) > MAX_API_QUERY || !query_chars_ok(query))) {
        finish_proxy_error(request, "api_query_denied");
        return;
    }
    const char *method = webkit_uri_scheme_request_get_http_method(request);
    if (method == NULL || method[0] == '\0') {
        method = "GET";
    }
    if (!method_ok(method)) {
        finish_proxy_error(request, "api_method_denied");
        return;
    }
    char *url = query != NULL && query[0] != '\0'
        ? g_strdup_printf("%s%s?%s", API_HOST, path, query)
        : g_strdup_printf("%s%s", API_HOST, path);
    SoupMessage *message = soup_message_new(method, url);
    g_free(url);
    if (message == NULL) {
        finish_proxy_error(request, "api_url_denied");
        return;
    }
    SoupMessageHeaders *headers = soup_message_get_request_headers(message);
    soup_message_headers_replace(headers, "Origin", API_HOST);
    soup_message_headers_replace(headers, "Referer", API_HOST "/");
    SoupMessageHeaders *incoming = webkit_uri_scheme_request_get_http_headers(request);
    if (incoming != NULL) {
        soup_message_headers_foreach(incoming, copy_request_header, headers);
    }
    /* Native complete never posts /auth/sso/callback (code already spent).
     * First /api hop must write the fofline sid: XOR/replace still needs
     * the living confirm header even when the binding PEP already allows
     * replacement. Plant it here so the pack does not have to. */
    soup_message_headers_replace(headers, "X-SSO-Replace-Confirm", "1");
    /* MusicPlayer / WebKit media send Range probes. A 206 slice on
     * fofline-spa: is a successful XHR of a few KB, then the spinner. */
    gboolean media_range = FALSE;
    if (path_is_content_download(path)) {
        media_range = incoming != NULL
            && soup_message_headers_get_one(incoming, "Range") != NULL;
        soup_message_headers_remove(headers, "Range");
    }
    GError *error = NULL;
    GInputStream *body_stream = webkit_uri_scheme_request_get_http_body(request);
    GBytes *body = read_body_bounded(body_stream, &error);
    if (body_stream != NULL) {
        g_object_unref(body_stream);
    }
    if (error != NULL) {
        syslog(LOG_ERR, "fofline api proxy body failed");
        g_clear_error(&error);
        g_object_unref(message);
        finish_proxy_error(request, "api_body_denied");
        return;
    }
    if (body != NULL) {
        if (g_bytes_get_size(body) > 0U) {
            const char *content_type = incoming != NULL
                ? soup_message_headers_get_one(incoming, "Content-Type")
                : NULL;
            soup_message_set_request_body_from_bytes(
                message,
                content_type != NULL ? content_type : "application/octet-stream",
                body
            );
        }
        g_bytes_unref(body);
    }
    syslog(LOG_INFO, "fofline api proxy method=%s path=%s", method, path);
    ApiProxyJob *job = g_new0(ApiProxyJob, 1U);
    job->request = g_object_ref(request);
    job->message = message;
    job->path = g_strdup(path);
    job->content_download = path_is_content_download(path);
    job->media_range = media_range;
    {
        const char *origin = incoming != NULL
            ? soup_message_headers_get_one(incoming, "Origin")
            : NULL;
        if (origin != NULL && g_str_has_prefix(origin, "fofline-spa:")) {
            job->allow_origin = g_strdup(origin);
        } else {
            job->allow_origin = g_strdup("fofline-spa://localhost");
        }
    }
    job->hangup = g_strcmp0(method, "POST") == 0
        && empiier_fofline_video_call_end_path(path);
    soup_session_send_and_read_async(
        api_session(job->content_download),
        message,
        G_PRIORITY_DEFAULT,
        NULL,
        proxy_finished,
        job
    );
}
