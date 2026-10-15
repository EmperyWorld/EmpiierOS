#define _GNU_SOURCE

/* LinkornEW console window, built from this source.
 * Accepts only `--title=LinkornEW`. Does not open a getty. */

#include <errno.h>
#include <errno.h>
#include <fcntl.h>
#include <gtk/gtk.h>
#include <pty.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>

static GtkTextBuffer *screen_buffer;
static int master_fd = -1;
static int escape = 0;

static void append_bytes(const char *bytes, gsize length) {
    GtkTextIter end;
    for (gsize i = 0; i < length; i++) {
        unsigned char ch = (unsigned char)bytes[i];
        if (escape) {
            if ((ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')) {
                escape = 0;
            }
            continue;
        }
        if (ch == 0x1b) {
            escape = 1;
            continue;
        }
        gtk_text_buffer_get_end_iter(screen_buffer, &end);
        if (ch == '\r') {
            continue;
        }
        if (ch == '\b' || ch == 127) {
            GtkTextIter start = end;
            if (gtk_text_iter_backward_char(&start)) {
                gtk_text_buffer_delete(screen_buffer, &start, &end);
            }
            continue;
        }
        char text[2] = {(char)ch, 0};
        if (ch == '\n' || ch == '\t' || ch >= 32) {
            gtk_text_buffer_insert(screen_buffer, &end, ch == '\n' ? "\n" : text, -1);
        }
    }
    GtkTextIter tail;
    gtk_text_buffer_get_end_iter(screen_buffer, &tail);
    gtk_text_view_scroll_to_iter(GTK_TEXT_VIEW(g_object_get_data(
        G_OBJECT(screen_buffer), "view")), &tail, 0.0, FALSE, 0.0, 1.0);
}

static gboolean read_master(GIOChannel *channel, GIOCondition condition, gpointer user_data) {
    (void)user_data;
    if ((condition & (G_IO_HUP | G_IO_ERR)) != 0) {
        gtk_main_quit();
        return FALSE;
    }
    char buf[512];
    gsize read_n = 0;
    GError *error = NULL;
    GIOStatus status = g_io_channel_read_chars(channel, buf, sizeof buf, &read_n, &error);
    if (error != NULL) {
        g_error_free(error);
        return TRUE;
    }
    if (read_n > 0) {
        append_bytes(buf, read_n);
    }
    if (status == G_IO_STATUS_EOF) {
        gtk_main_quit();
        return FALSE;
    }
    return TRUE;
}

static gboolean on_key(GtkWidget *widget, GdkEventKey *event, gpointer user_data) {
    (void)widget;
    (void)user_data;
    char out[8];
    gsize len = 0;
    if (event->keyval == GDK_KEY_Return || event->keyval == GDK_KEY_KP_Enter) {
        out[0] = '\n';
        len = 1;
    } else if (event->keyval == GDK_KEY_BackSpace) {
        out[0] = 127;
        len = 1;
    } else {
        guint32 unicode = gdk_keyval_to_unicode(event->keyval);
        if (unicode >= 32 && unicode < 127) {
            out[0] = (char)unicode;
            len = 1;
        }
    }
    if (len > 0 && master_fd >= 0) {
        if (write(master_fd, out, len) < 0) {
            return TRUE;
        }
    }
    return TRUE;
}

static int take_tty(int fd) {
    if (setsid() < 0 && errno != EPERM) {
        return -1;
    }
    if (ioctl(fd, TIOCSCTTY, 0) < 0) {
        return -1;
    }
    if (dup2(fd, STDIN_FILENO) < 0 || dup2(fd, STDOUT_FILENO) < 0 || dup2(fd, STDERR_FILENO) < 0) {
        return -1;
    }
    if (fd > STDERR_FILENO) {
        close(fd);
    }
    return 0;
}

static int args_ok(int argc, char **argv) {
    return argc == 2 && strcmp(argv[1], "--title=LinkornEW") == 0;
}

static void start_shell(void) {
    int slave = -1;
    struct termios tio;
    memset(&tio, 0, sizeof tio);
    tio.c_iflag = ICRNL;
    tio.c_oflag = OPOST | ONLCR;
    tio.c_cflag = CS8 | CREAD;
    tio.c_lflag = ICANON | ECHO | ECHOE | ISIG;
    tio.c_cc[VMIN] = 1;
    tio.c_cc[VTIME] = 0;
    if (openpty(&master_fd, &slave, NULL, &tio, NULL) != 0) {
        fprintf(stderr, "linkornew-console: openpty failed\n");
        exit(1);
    }
    pid_t child = fork();
    if (child < 0) {
        fprintf(stderr, "linkornew-console: fork failed\n");
        exit(1);
    }
    if (child == 0) {
        close(master_fd);
        if (take_tty(slave) != 0) {
            _exit(1);
        }
        setenv("TERM", "dumb", 1);
        setenv("PS1", "linkorn# ", 1);
        execl("/bin/sh", "sh", "-i", (char *)NULL);
        _exit(127);
    }
    close(slave);
    int flags = fcntl(master_fd, F_GETFL, 0);
    if (flags >= 0) {
        fcntl(master_fd, F_SETFL, flags | O_NONBLOCK);
    }
}

int main(int argc, char **argv) {
    if (!args_ok(argc, argv)) {
        fprintf(stderr, "linkornew-console: refused\n");
        return 1;
    }
    gtk_init(&argc, &argv);
    start_shell();
    GtkWidget *window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(window), "LinkornEW");
    gtk_window_set_default_size(GTK_WINDOW(window), 960, 600);
    GtkWidget *view = gtk_text_view_new();
    gtk_text_view_set_editable(GTK_TEXT_VIEW(view), FALSE);
    gtk_text_view_set_cursor_visible(GTK_TEXT_VIEW(view), TRUE);
    screen_buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(view));
    g_object_set_data(G_OBJECT(screen_buffer), "view", view);
    GtkWidget *scroll = gtk_scrolled_window_new(NULL, NULL);
    gtk_container_add(GTK_CONTAINER(scroll), view);
    gtk_container_add(GTK_CONTAINER(window), scroll);
    g_signal_connect(view, "key-press-event", G_CALLBACK(on_key), NULL);
    g_signal_connect(window, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    GIOChannel *channel = g_io_channel_unix_new(master_fd);
    g_io_channel_set_encoding(channel, NULL, NULL);
    g_io_channel_set_flags(channel, G_IO_FLAG_NONBLOCK, NULL);
    g_io_add_watch(channel, G_IO_IN | G_IO_HUP | G_IO_ERR, read_master, NULL);
    gtk_widget_show_all(window);
    gtk_widget_grab_focus(view);
    gtk_main();
    return 0;
}
