# Staff seat window and console. Not invoked by the FoFline image packager.
WEBKIT_PC ?= webkit2gtk-4.1

.PHONY: all
all: linkornew-seat console command-post/cp-deck-serve command-post/cp-deck-stub

linkornew-seat: linkornew-seat.c
	gcc -Wall -Wextra -O2 -o $@ $< $$(pkg-config --cflags --libs $(WEBKIT_PC) gtk+-3.0)

console: console.c
	gcc -Wall -Wextra -O2 -o $@ $< $$(pkg-config --cflags --libs gtk+-3.0) -lutil

command-post/cp-deck-serve: command-post/cp-deck-serve.c
	gcc -Wall -Wextra -O2 -o $@ $<

command-post/cp-deck-stub: command-post/cp-deck-stub.c
	gcc -Wall -Wextra -O2 -o $@ $<
