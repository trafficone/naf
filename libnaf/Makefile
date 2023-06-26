.POSIX:
.SUFFIXES:

TARGET = lib/libnaf.so
HFILE = lib/libnaf.h

CC ?= gcc
ifeq ($(origin CC),default)
CC = gcc
endif

LIBFLAGS = -shared 
CFLAGS = -std=gnu99 -Wall -Wextra -O3 -march=native -ffast-math -c -fPIC -I../zstd/lib
LDFLAGS = ../zstd/lib/libzstd.a

.PHONY: default all clean install uninstall

SOURCES = $(wildcard src/*)

default: $(TARGET)

$(TARGET): naf.o
	$(CC) $< $(LIBFLAGS) -o $@

naf.o: $(SOURCES)
	$(CC) $(CFLAGS) -o $@ src/libnaf.c $(LDFLAGS)

all: default

clean:
	-rm -f $(TARGET)

install: $(TARGET)
	mkdir -p $(DESTDIR)$(prefix)/include/naflib
	mkdir -p $(DESTDIR)$(prefix)/lib
	cp -f $(HFILE) $(DESTDIR)$(prefix)/include/naflib
	cp -f $(TARGET) $(DESTDIR)$(prefix)/lib

uninstall:
	-rm -f $(DESTDIR)$(prefix)/include/naflib/$(HFILE)
	-rm -f $(DESTDIR)$(prefix)/lib/$(TARGET)
