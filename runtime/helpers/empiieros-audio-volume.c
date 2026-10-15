#define _POSIX_C_SOURCE 200809L
#include "hda_speaker_amp.h"

#include <alsa/asoundlib.h>
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define REQUEST_PATH "/run/empiieros/audio-volume.request"
#define MIN_PERCENT 10
#define MAX_PERCENT 100

static const char *const ALLOWED[] = {
    "Speaker",
    "Headphone",
    "Master",
    "PCM",
    NULL};

static int allowed_name(const char *name) {
    size_t i;
    if (name == NULL || name[0] == '\0') {
        return 1;
    }
    for (i = 0; ALLOWED[i] != NULL; i++) {
        if (strcmp(name, ALLOWED[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

static int parse_request(char *name, size_t name_len, unsigned *percent) {
    FILE *file;
    char line[256];
    int got_percent = 0;

    name[0] = '\0';
    file = fopen(REQUEST_PATH, "r");
    if (file == NULL) {
        fprintf(stderr, "audio-volume: request missing\n");
        return -1;
    }
    while (fgets(line, sizeof(line), file) != NULL) {
        char *nl = strchr(line, '\n');
        char *eq;
        if (nl != NULL) {
            *nl = '\0';
        }
        if (line[0] == '\0' || line[0] == '#') {
            continue;
        }
        eq = strchr(line, '=');
        if (eq == NULL) {
            fclose(file);
            fprintf(stderr, "audio-volume: malformed request\n");
            return -1;
        }
        *eq = '\0';
        if (strcmp(line, "name") == 0) {
            if (strlen(eq + 1) >= name_len) {
                fclose(file);
                fprintf(stderr, "audio-volume: name too long\n");
                return -1;
            }
            memcpy(name, eq + 1, strlen(eq + 1) + 1);
        } else if (strcmp(line, "percent") == 0) {
            char *end = NULL;
            unsigned long value = strtoul(eq + 1, &end, 10);
            if (end == eq + 1 || *end != '\0' || value < MIN_PERCENT || value > MAX_PERCENT) {
                fclose(file);
                fprintf(stderr, "audio-volume: percent out of range\n");
                return -1;
            }
            *percent = (unsigned)value;
            got_percent = 1;
        } else {
            fclose(file);
            fprintf(stderr, "audio-volume: unknown field\n");
            return -1;
        }
    }
    fclose(file);
    if (!got_percent) {
        fprintf(stderr, "audio-volume: percent required\n");
        return -1;
    }
    if (!allowed_name(name)) {
        fprintf(stderr, "audio-volume: control name not allowlisted\n");
        return -1;
    }
    return 0;
}

static int get_elem(snd_mixer_elem_t *elem, long *observed) {
    long min = 0;
    long max = 0;
    long left = 0;

    if (!snd_mixer_selem_has_playback_volume(elem)) {
        return 0;
    }
    if (snd_mixer_selem_get_playback_volume_range(elem, &min, &max) < 0 || max <= min) {
        return -1;
    }
    if (snd_mixer_selem_get_playback_volume(elem, SND_MIXER_SCHN_FRONT_LEFT, &left) < 0) {
        return -1;
    }
    *observed = ((left - min) * 100) / (max - min);
    return 1;
}

static int set_elem(snd_mixer_elem_t *elem, unsigned percent, long *observed) {
    long min = 0;
    long max = 0;
    long value;
    long left = 0;

    if (!snd_mixer_selem_has_playback_volume(elem)) {
        return 0;
    }
    if (snd_mixer_selem_get_playback_volume_range(elem, &min, &max) < 0 || max <= min) {
        return -1;
    }
    value = min + (((long)percent * (max - min)) / 100);
    if (snd_mixer_selem_set_playback_volume_all(elem, value) < 0) {
        return -1;
    }
    if (snd_mixer_selem_has_playback_switch(elem)
        && snd_mixer_selem_set_playback_switch_all(elem, 1) < 0) {
        return -1;
    }
    if (snd_mixer_selem_get_playback_volume(elem, SND_MIXER_SCHN_FRONT_LEFT, &left) < 0) {
        return -1;
    }
    *observed = ((left - min) * 100) / (max - min);
    return 1;
}

static int observe_only(void) {
    snd_mixer_t *mixer = NULL;
    snd_mixer_elem_t *elem;
    long observed = -1;
    const char *used = "";
    int found = 0;

    if (snd_mixer_open(&mixer, 0) < 0 || snd_mixer_attach(mixer, "default") < 0
        || snd_mixer_selem_register(mixer, NULL, NULL) < 0 || snd_mixer_load(mixer) < 0) {
        fprintf(stderr, "audio-volume: mixer unavailable\n");
        if (mixer != NULL) {
            snd_mixer_close(mixer);
        }
        return 1;
    }
    empiieros_hda_enable_speaker_amp();
    for (elem = snd_mixer_first_elem(mixer); elem != NULL; elem = snd_mixer_elem_next(elem)) {
        const char *elem_name = snd_mixer_selem_get_name(elem);
        long current = -1;
        if (!allowed_name(elem_name)) {
            continue;
        }
        if (get_elem(elem, &current) != 1) {
            continue;
        }
        if (!found || strcmp(elem_name, "Speaker") == 0) {
            used = elem_name;
            observed = current;
        }
        found = 1;
    }
    if (!found) {
        fprintf(stderr, "audio-volume: no allowlisted playback control\n");
        snd_mixer_close(mixer);
        return 1;
    }
    printf("name=%s\nobserved_percent=%ld\n", used, observed);
    snd_mixer_close(mixer);
    return 0;
}

int main(int argc, char **argv) {
    char name[96];
    unsigned percent = 0;
    snd_mixer_t *mixer = NULL;
    snd_mixer_selem_id_t *sid = NULL;
    snd_mixer_elem_t *elem;
    int applied = 0;
    long observed = -1;
    const char *used = "";

    if (argc == 2 && strcmp(argv[1], "--observe") == 0) {
        return observe_only();
    }
    if (argc != 1) {
        fprintf(stderr, "usage: empiieros-audio-volume [--observe]\n");
        return 2;
    }
    if (parse_request(name, sizeof(name), &percent) != 0) {
        return 1;
    }
    if (snd_mixer_open(&mixer, 0) < 0 || snd_mixer_attach(mixer, "default") < 0
        || snd_mixer_selem_register(mixer, NULL, NULL) < 0 || snd_mixer_load(mixer) < 0) {
        fprintf(stderr, "audio-volume: mixer unavailable\n");
        if (mixer != NULL) {
            snd_mixer_close(mixer);
        }
        return 1;
    }
    empiieros_hda_enable_speaker_amp();

    if (name[0] != '\0') {
        if (snd_mixer_selem_id_malloc(&sid) < 0) {
            fprintf(stderr, "audio-volume: selem id alloc failed\n");
            snd_mixer_close(mixer);
            return 1;
        }
        snd_mixer_selem_id_set_index(sid, 0);
        snd_mixer_selem_id_set_name(sid, name);
        elem = snd_mixer_find_selem(mixer, sid);
        snd_mixer_selem_id_free(sid);
        if (elem == NULL || set_elem(elem, percent, &observed) != 1) {
            fprintf(stderr, "audio-volume: control not applied\n");
            snd_mixer_close(mixer);
            return 1;
        }
        used = name;
        applied = 1;
    } else {
        for (elem = snd_mixer_first_elem(mixer); elem != NULL; elem = snd_mixer_elem_next(elem)) {
            const char *elem_name = snd_mixer_selem_get_name(elem);
            if (!allowed_name(elem_name)) {
                continue;
            }
            if (set_elem(elem, percent, &observed) == 1) {
                if (applied == 0 || strcmp(elem_name, "Speaker") == 0) {
                    used = elem_name;
                }
                applied = 1;
            }
        }
        if (!applied) {
            fprintf(stderr, "audio-volume: no allowlisted playback control\n");
            snd_mixer_close(mixer);
            return 1;
        }
    }

    {
        FILE *result = fopen("/run/empiieros/audio-volume.result", "w");
        if (result == NULL) {
            fprintf(stderr, "audio-volume: result write failed\n");
            snd_mixer_close(mixer);
            return 1;
        }
        fprintf(
            result,
            "name=%s\npercent=%u\nobserved_percent=%ld\n",
            used,
            percent,
            observed);
        if (fclose(result) != 0) {
            snd_mixer_close(mixer);
            return 1;
        }
    }
    printf("name=%s\npercent=%u\nobserved_percent=%ld\n", used, percent, observed);
    snd_mixer_close(mixer);
    return 0;
}
