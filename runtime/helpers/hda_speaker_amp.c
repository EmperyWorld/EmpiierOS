#define _POSIX_C_SOURCE 200809L
#include "hda_speaker_amp.h"

#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>

#define HDA_VENDOR_SYSFS "/sys/class/sound/hwC0D0/vendor_id"
#define HDA_SUBSYS_SYSFS "/sys/class/sound/hwC0D0/subsystem_id"
#define HDA_HWDEP "/dev/snd/hwC0D0"
#define HDA_VENDOR_ALC3204 0x10ec0236u
#define HDA_SUBSYS_DELL_09C1 0x102809c1u
#define HDA_AFG 0x01u
#define HDA_GPIO_MASK 0x07u
#define HDA_VERB_GET_GPIO_DATA 0xf15u
#define HDA_VERB_GET_GPIO_MASK 0xf16u
#define HDA_VERB_GET_GPIO_DIR 0xf17u
#define HDA_VERB_SET_GPIO_DATA 0x715u
#define HDA_VERB_SET_GPIO_MASK 0x716u
#define HDA_VERB_SET_GPIO_DIR 0x717u
#define HDA_VERB(nid, verb, param) (((nid) << 24) | ((verb) << 8) | (param))
#define HDA_IOCTL_VERB_WRITE _IOWR('H', 0x11, struct hda_verb_ioctl)

struct hda_verb_ioctl {
    uint32_t verb;
    uint32_t res;
};

static int read_hex_sysfs(const char *path, unsigned *out) {
    FILE *file;
    char line[32];
    char *end = NULL;
    unsigned long value;

    file = fopen(path, "r");
    if (file == NULL) {
        return -1;
    }
    if (fgets(line, sizeof(line), file) == NULL) {
        fclose(file);
        return -1;
    }
    fclose(file);
    value = strtoul(line, &end, 0);
    if (end == line || (*end != '\0' && *end != '\n')) {
        return -1;
    }
    *out = (unsigned)value;
    return 0;
}

static int verb_io(int fd, unsigned nid, unsigned verb, unsigned param, uint32_t *res) {
    struct hda_verb_ioctl io;

    io.verb = HDA_VERB(nid, verb, param);
    io.res = 0;
    if (ioctl(fd, HDA_IOCTL_VERB_WRITE, &io) < 0) {
        return -1;
    }
    if (res != NULL) {
        *res = io.res;
    }
    return 0;
}

void empiieros_hda_enable_speaker_amp(void) {
    unsigned vendor = 0;
    unsigned subsys = 0;
    int fd;
    uint32_t mask = 0;
    uint32_t dir = 0;
    uint32_t data = 0;

    if (read_hex_sysfs(HDA_VENDOR_SYSFS, &vendor) < 0
        || read_hex_sysfs(HDA_SUBSYS_SYSFS, &subsys) < 0) {
        return;
    }
    if (vendor != HDA_VENDOR_ALC3204 || subsys != HDA_SUBSYS_DELL_09C1) {
        return;
    }

    fd = open(HDA_HWDEP, O_RDWR);
    if (fd < 0) {
        fprintf(stderr, "audio-volume: speaker amp hwdep unavailable\n");
        return;
    }
    if (verb_io(fd, HDA_AFG, HDA_VERB_GET_GPIO_MASK, 0, &mask) < 0
        || verb_io(fd, HDA_AFG, HDA_VERB_GET_GPIO_DIR, 0, &dir) < 0
        || verb_io(fd, HDA_AFG, HDA_VERB_GET_GPIO_DATA, 0, &data) < 0) {
        fprintf(stderr, "audio-volume: speaker amp gpio read failed\n");
        close(fd);
        return;
    }
    if (mask == HDA_GPIO_MASK && dir == HDA_GPIO_MASK && data == HDA_GPIO_MASK) {
        close(fd);
        return;
    }
    if (verb_io(fd, HDA_AFG, HDA_VERB_SET_GPIO_MASK, HDA_GPIO_MASK, NULL) < 0
        || verb_io(fd, HDA_AFG, HDA_VERB_SET_GPIO_DIR, HDA_GPIO_MASK, NULL) < 0
        || verb_io(fd, HDA_AFG, HDA_VERB_SET_GPIO_DATA, HDA_GPIO_MASK, NULL) < 0
        || verb_io(fd, HDA_AFG, HDA_VERB_GET_GPIO_DATA, 0, &data) < 0
        || data != HDA_GPIO_MASK) {
        fprintf(stderr, "audio-volume: speaker amp gpio enable failed\n");
        close(fd);
        return;
    }
    fprintf(stderr, "audio-volume: speaker amp gpio enabled\n");
    close(fd);
}
