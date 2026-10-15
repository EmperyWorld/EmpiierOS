#define _DEFAULT_SOURCE
#define _POSIX_C_SOURCE 200809L

/*
 * LinkornEW Command Deck serve — L5.
 * Bind ONLY 127.0.0.1:4191.
 * Serves planted Command Deck static files; proxies /api|/health|/readyz
 * to http://127.0.0.1:8088 (local controller). Not MATCH. No tokens here.
 */

#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <unistd.h>

#define DECK_PORT 4191
#define CTRL_PORT 8088
#define ROOT_DEFAULT "/usr/lib/linkornew/command-deck"
#define BUF 65536
#define HDR_MAX 16384

static const char FALLBACK_HTML[] =
    "<!doctype html><html lang=\"en\"><head><meta charset=\"UTF-8\"/>"
    "<title>LinkornEW Command Post</title></head><body>"
    "<h1>LinkornEW</h1>"
    "<p>Command Deck is not planted yet. Expected static tree under "
    "<code>/usr/lib/linkornew/command-deck/</code> (index.html + assets).</p>"
    "<p>Bind is 127.0.0.1:4191. Proxy target is 127.0.0.1:8088. "
    "Generation-only until the real controller unit runs.</p>"
    "</body></html>";

static volatile sig_atomic_t g_stop = 0;

static void on_signal(int sig) {
    (void)sig;
    g_stop = 1;
}

static int is_proxy_path(const char *path) {
    return strncmp(path, "/api", 4) == 0
        || strcmp(path, "/health") == 0
        || strcmp(path, "/readyz") == 0
        || strncmp(path, "/health?", 8) == 0
        || strncmp(path, "/readyz?", 8) == 0;
}

static const char *guess_ctype(const char *path) {
    const char *dot = strrchr(path, '.');
    if (!dot) {
        return "application/octet-stream";
    }
    if (strcmp(dot, ".html") == 0) {
        return "text/html; charset=utf-8";
    }
    if (strcmp(dot, ".js") == 0) {
        return "text/javascript; charset=utf-8";
    }
    if (strcmp(dot, ".css") == 0) {
        return "text/css; charset=utf-8";
    }
    if (strcmp(dot, ".svg") == 0) {
        return "image/svg+xml";
    }
    if (strcmp(dot, ".json") == 0) {
        return "application/json";
    }
    if (strcmp(dot, ".png") == 0) {
        return "image/png";
    }
    if (strcmp(dot, ".woff2") == 0) {
        return "font/woff2";
    }
    if (strcmp(dot, ".ico") == 0) {
        return "image/x-icon";
    }
    return "application/octet-stream";
}

static void send_all(int fd, const void *buf, size_t len) {
    const char *p = (const char *)buf;
    while (len > 0) {
        ssize_t n = send(fd, p, len, 0);
        if (n <= 0) {
            return;
        }
        p += (size_t)n;
        len -= (size_t)n;
    }
}

static void send_simple(int fd, int code, const char *ctype, const void *body, size_t len) {
    char hdr[256];
    const char *reason = code == 200 ? "OK" : code == 404 ? "Not Found" : "Bad Gateway";
    int n = snprintf(
        hdr,
        sizeof hdr,
        "HTTP/1.1 %d %s\r\nContent-Type: %s\r\nContent-Length: %zu\r\n"
        "Connection: close\r\nCache-Control: no-store\r\n\r\n",
        code,
        reason,
        ctype,
        len
    );
    if (n > 0) {
        send_all(fd, hdr, (size_t)n);
        if (body && len) {
            send_all(fd, body, len);
        }
    }
}

static int under_root(const char *root, char *out, size_t out_sz, const char *url_path) {
    char joined[4096];
    char resolved_root[4096];
    char resolved_file[4096];
    const char *rel = url_path;
    if (rel[0] == '/') {
        rel++;
    }
    if (rel[0] == '\0') {
        rel = "index.html";
    }
    if (strstr(rel, "..") != NULL) {
        return -1;
    }
    if (snprintf(joined, sizeof joined, "%s/%s", root, rel) >= (int)sizeof joined) {
        return -1;
    }
    if (!realpath(root, resolved_root)) {
        return -1;
    }
    if (!realpath(joined, resolved_file)) {
        return -1;
    }
    size_t root_len = strlen(resolved_root);
    if (strncmp(resolved_file, resolved_root, root_len) != 0) {
        return -1;
    }
    if (resolved_file[root_len] != '\0' && resolved_file[root_len] != '/') {
        return -1;
    }
    if (strlen(resolved_file) + 1 > out_sz) {
        return -1;
    }
    memcpy(out, resolved_file, strlen(resolved_file) + 1);
    return 0;
}

static void serve_file(int fd_client, const char *path) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        send_simple(fd_client, 404, "text/plain; charset=utf-8", "not found\n", 10);
        return;
    }
    struct stat st;
    if (fstat(fd, &st) != 0 || !S_ISREG(st.st_mode) || st.st_size < 0) {
        close(fd);
        send_simple(fd_client, 404, "text/plain; charset=utf-8", "not found\n", 10);
        return;
    }
    char hdr[320];
    int n = snprintf(
        hdr,
        sizeof hdr,
        "HTTP/1.1 200 OK\r\nContent-Type: %s\r\nContent-Length: %lld\r\n"
        "Connection: close\r\nCache-Control: no-store\r\n\r\n",
        guess_ctype(path),
        (long long)st.st_size
    );
    if (n > 0) {
        send_all(fd_client, hdr, (size_t)n);
    }
    char buf[BUF];
    for (;;) {
        ssize_t r = read(fd, buf, sizeof buf);
        if (r <= 0) {
            break;
        }
        send_all(fd_client, buf, (size_t)r);
    }
    close(fd);
}

static void serve_static(int client, const char *root, const char *url_path) {
    char file[4096];
    char index_path[4096];
    if (under_root(root, file, sizeof file, url_path) == 0) {
        serve_file(client, file);
        return;
    }
    /* SPA fallback */
    if (under_root(root, index_path, sizeof index_path, "/index.html") == 0) {
        serve_file(client, index_path);
        return;
    }
    send_simple(
        client,
        200,
        "text/html; charset=utf-8",
        FALLBACK_HTML,
        sizeof FALLBACK_HTML - 1
    );
}

static int connect_controller(void) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        return -1;
    }
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof addr);
    addr.sin_family = AF_INET;
    addr.sin_port = htons(CTRL_PORT);
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    if (connect(fd, (struct sockaddr *)&addr, sizeof addr) != 0) {
        close(fd);
        return -1;
    }
    return fd;
}

static const char *find_header_ci(const char *buf, const char *end, const char *name) {
    size_t nlen = strlen(name);
    const char *p = buf;
    while (p + nlen + 1 < end) {
        size_t i = 0;
        while (i < nlen) {
            char a = p[i];
            char b = name[i];
            if (a >= 'A' && a <= 'Z') {
                a = (char)(a - 'A' + 'a');
            }
            if (b >= 'A' && b <= 'Z') {
                b = (char)(b - 'A' + 'a');
            }
            if (a != b) {
                break;
            }
            i++;
        }
        if (i == nlen && p[nlen] == ':') {
            return p;
        }
        p++;
    }
    return NULL;
}

static int rewrite_connection_close(const char *req, size_t req_len, char *out, size_t out_sz, size_t *out_len) {
    const char *hdr_end = strstr(req, "\r\n\r\n");
    if (!hdr_end || hdr_end < req) {
        return -1;
    }
    size_t header_bytes = (size_t)(hdr_end - req);
    size_t body_off = header_bytes + 4;
    size_t body_len = req_len > body_off ? req_len - body_off : 0;

    /* First request line */
    const char *line_end = strstr(req, "\r\n");
    if (!line_end) {
        return -1;
    }
    size_t line_len = (size_t)(line_end - req) + 2;
    if (line_len >= out_sz) {
        return -1;
    }
    memcpy(out, req, line_len);
    size_t o = line_len;

    /* Copy headers except Connection / Keep-Alive; force Connection: close */
    const char *p = line_end + 2;
    while (p < hdr_end) {
        const char *nl = strstr(p, "\r\n");
        if (!nl || nl > hdr_end) {
            break;
        }
        size_t hlen = (size_t)(nl - p);
        int skip = 0;
        if (hlen >= 10 && strncasecmp(p, "Connection", 10) == 0) {
            skip = 1;
        } else if (hlen >= 10 && strncasecmp(p, "Keep-Alive", 10) == 0) {
            skip = 1;
        }
        if (!skip) {
            if (o + hlen + 2 >= out_sz) {
                return -1;
            }
            memcpy(out + o, p, hlen + 2);
            o += hlen + 2;
        }
        p = nl + 2;
    }
    const char *force = "Connection: close\r\n\r\n";
    size_t fl = strlen(force);
    if (o + fl + body_len >= out_sz) {
        return -1;
    }
    memcpy(out + o, force, fl);
    o += fl;
    if (body_len) {
        memcpy(out + o, req + body_off, body_len);
        o += body_len;
    }
    *out_len = o;
    return 0;
}

static void proxy_to_controller(int client, const char *req, size_t req_len) {
    int upstream = connect_controller();
    if (upstream < 0) {
        const char *msg =
            "{\"detail\":\"controller unreachable on 127.0.0.1:8088\"}\n";
        send_simple(client, 502, "application/json", msg, strlen(msg));
        return;
    }

    char rewritten[HDR_MAX + 64];
    size_t rewritten_len = 0;
    const char *forward = req;
    size_t forward_len = req_len;
    if (rewrite_connection_close(req, req_len, rewritten, sizeof rewritten, &rewritten_len) == 0) {
        forward = rewritten;
        forward_len = rewritten_len;
    }
    send_all(upstream, forward, forward_len);

    const char *hdr_end = strstr(req, "\r\n\r\n");
    if (hdr_end) {
        size_t header_bytes = (size_t)(hdr_end - req) + 4;
        size_t already = req_len > header_bytes ? req_len - header_bytes : 0;
        long content_length = -1;
        const char *cl = find_header_ci(req, hdr_end, "content-length");
        if (cl) {
            const char *v = strchr(cl, ':');
            if (v) {
                content_length = strtol(v + 1, NULL, 10);
            }
        }
        if (content_length > 0 && (long)already < content_length) {
            long need = content_length - (long)already;
            while (need > 0) {
                char chunk[BUF];
                size_t take = (size_t)need < sizeof chunk ? (size_t)need : sizeof chunk;
                ssize_t r = recv(client, chunk, take, 0);
                if (r <= 0) {
                    break;
                }
                send_all(upstream, chunk, (size_t)r);
                need -= (long)r;
            }
        }
    }

    char buf[BUF];
    for (;;) {
        ssize_t r = recv(upstream, buf, sizeof buf, 0);
        if (r <= 0) {
            break;
        }
        send_all(client, buf, (size_t)r);
    }
    close(upstream);
}

static void handle_client(int client, const char *root) {
    char req[HDR_MAX];
    size_t got = 0;
    while (got < sizeof req - 1) {
        ssize_t r = recv(client, req + got, sizeof req - 1 - got, 0);
        if (r <= 0) {
            return;
        }
        got += (size_t)r;
        req[got] = '\0';
        if (strstr(req, "\r\n\r\n") != NULL) {
            break;
        }
    }
    char method[16];
    char target[2048];
    if (sscanf(req, "%15s %2047s", method, target) != 2) {
        send_simple(client, 400, "text/plain; charset=utf-8", "bad request\n", 12);
        return;
    }
    char path[2048];
    snprintf(path, sizeof path, "%s", target);
    char *q = strchr(path, '?');
    if (q) {
        *q = '\0';
    }

    if (is_proxy_path(path)) {
        if (strcmp(method, "GET") != 0
            && strcmp(method, "POST") != 0
            && strcmp(method, "PUT") != 0
            && strcmp(method, "PATCH") != 0
            && strcmp(method, "DELETE") != 0
            && strcmp(method, "HEAD") != 0
            && strcmp(method, "OPTIONS") != 0) {
            send_simple(client, 405, "text/plain; charset=utf-8", "method\n", 7);
            return;
        }
        proxy_to_controller(client, req, got);
        return;
    }

    if (strcmp(method, "GET") != 0 && strcmp(method, "HEAD") != 0) {
        send_simple(client, 405, "text/plain; charset=utf-8", "method\n", 7);
        return;
    }
    serve_static(client, root, path);
}

int main(void) {
    const char *root = getenv("LINKORNEW_DECK_ROOT");
    if (!root || root[0] == '\0') {
        root = ROOT_DEFAULT;
    }

    (void)signal(SIGPIPE, SIG_IGN);
    (void)signal(SIGINT, on_signal);
    (void)signal(SIGTERM, on_signal);

    int server = socket(AF_INET, SOCK_STREAM, 0);
    if (server < 0) {
        perror("linkornew-cp-deck: socket");
        return 1;
    }
    int on = 1;
    (void)setsockopt(server, SOL_SOCKET, SO_REUSEADDR, &on, sizeof on);
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof addr);
    addr.sin_family = AF_INET;
    addr.sin_port = htons(DECK_PORT);
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    if (bind(server, (struct sockaddr *)&addr, sizeof addr) != 0) {
        perror("linkornew-cp-deck: bind");
        close(server);
        return 1;
    }
    if (listen(server, 32) != 0) {
        perror("linkornew-cp-deck: listen");
        close(server);
        return 1;
    }
    fprintf(
        stderr,
        "linkornew-cp-deck: 127.0.0.1:%d root=%s proxy=127.0.0.1:%d\n",
        DECK_PORT,
        root,
        CTRL_PORT
    );

    while (!g_stop) {
        int client = accept(server, NULL, NULL);
        if (client < 0) {
            if (errno == EINTR) {
                continue;
            }
            break;
        }
        handle_client(client, root);
        close(client);
    }
    close(server);
    return 0;
}
