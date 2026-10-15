#define _POSIX_C_SOURCE 200809L

/* Listen only on 127.0.0.1:4191. Not the real Command Post. Loopback only.
 * Historical placeholder kept intentionally (do not delete). Prefer cp-deck-serve
 * when a planted Deck tree exists. */

#include <arpa/inet.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

static volatile sig_atomic_t g_stop = 0;

static void on_signal(int sig) {
    (void)sig;
    g_stop = 1;
}

static const char BODY[] =
    "<!doctype html><html lang=\"en\"><head><meta charset=\"UTF-8\"/>"
    "<title>LinkornEW Command Post stub</title></head><body>"
    "<h1>LinkornEW</h1>"
    "<p>Placeholder deck stub. Not the real Command Post. Not MATCH. "
    "Bind is 127.0.0.1:4191.</p></body></html>";

int main(void) {
    (void)signal(SIGINT, on_signal);
    (void)signal(SIGTERM, on_signal);

    int server = socket(AF_INET, SOCK_STREAM, 0);
    if (server < 0) {
        perror("linkornew-cp-deck-stub: socket");
        return 1;
    }
    int on = 1;
    (void)setsockopt(server, SOL_SOCKET, SO_REUSEADDR, &on, sizeof on);
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof addr);
    addr.sin_family = AF_INET;
    addr.sin_port = htons(4191);
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    if (bind(server, (struct sockaddr *)&addr, sizeof addr) != 0) {
        perror("linkornew-cp-deck-stub: bind");
        close(server);
        return 1;
    }
    if (listen(server, 8) != 0) {
        perror("linkornew-cp-deck-stub: listen");
        close(server);
        return 1;
    }

    while (!g_stop) {
        int client = accept(server, NULL, NULL);
        if (client < 0) {
            continue;
        }
        char req[1024];
        (void)recv(client, req, sizeof req - 1, 0);
        char hdr[256];
        int n = snprintf(
            hdr,
            sizeof hdr,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n"
            "Content-Length: %zu\r\nConnection: close\r\n\r\n",
            sizeof BODY - 1
        );
        if (n > 0) {
            (void)send(client, hdr, (size_t)n, 0);
            (void)send(client, BODY, sizeof BODY - 1, 0);
        }
        close(client);
    }
    close(server);
    return 0;
}
