TARGET := condux
BINARY := $(TARGET)

LIBRARY := libcondux.a

BUILDDIR := build
SRCDIR := src

CFILES := \
	$(SRCDIR)/assets.c \
	$(SRCDIR)/collision.c \
	$(SRCDIR)/input.c \
	$(SRCDIR)/linalg.c \
	$(SRCDIR)/main.c \
	$(SRCDIR)/spline.c \
	$(SRCDIR)/state.c \
	$(SRCDIR)/vehicle.c

CFLAGS := -Wall -Oz -Iinclude -Ibuild

CC := gcc
AR := ar

LDFLAGS := -lm

PLATFORM ?= sdl

ERR =

ifeq ($(PLATFORM), sdl)
	LDFLAGS += -lSDL2 -lGL
	CFILES += $(SRCDIR)/platform/sdl.c
	RUN_COMMAND := ./$(TARGET)
	BUILD_COMMAND := rust/build-sdl.sh
else ifeq ($(PLATFORM), 3ds)
	CC := $(DEVKITARM)/bin/arm-none-eabi-gcc
	AR := $(DEVKITARM)/bin/arm-none-eabi-ar
	CFLAGS += \
		-mword-relocations \
		-ffunction-sections \
		-march=armv6k \
		-mtune=mpcore \
		-mfloat-abi=hard \
		-mtp=soft \
		-D__3DS__ \
		-I$(DEVKITPRO)/libctru/include \
		-L$(DEVKITPRO)/libctru/lib
	LDFLAGS += -specs=3dsx.specs -lcitro2d -lcitro3d -lctru -lm
	BINARY := $(BINARY).3dsx
	TARGET := $(BINARY)
	CFILES += $(SRCDIR)/platform/ctr.c
	RUN_COMMAND := citra-qt $(TARGET)
	BUILD_COMMAND := rust/build-3ds.sh
else
	ERR = $(error unknown platform: $(PLATFORM))
endif

OFILES := $(CFILES:$(SRCDIR)/%.c=$(BUILDDIR)/%.o)
DFILES := $(CFILES:$(SRCDIR)/%.c=$(BUILDDIR)/%.d)

$(ERR)

.PHONY: all clean run

all: $(TARGET)

%.dol: %.elf
	elf2dol $< $@

$(TARGET): $(LIBRARY) $(wildcard rust/src/*.rs)
	$(BUILD_COMMAND) $(TARGET)

$(BUILDDIR)/assets.o: $(BUILDDIR)/bundle.h
$(BUILDDIR)/assets.d: $(BUILDDIR)/bundle.h

$(BUILDDIR)/bundle.h: $(shell find assets)
	mkdir -p $(dir $@)
	python scripts/asset_bundler.py

$(LIBRARY): $(OFILES)
	$(AR) -rcs $@ $(OFILES)

$(BUILDDIR)/%.d: $(SRCDIR)/%.c
	mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -MM -MT $(<:$(SRCDIR)/%.c=$(BUILDDIR)/%.o) $< -MF $@

$(BUILDDIR)/%.o: $(SRCDIR)/%.c
	mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -rf $(BUILDDIR) $(TARGET) $(BINARY) $(LIBRARY)
	# cd rust && cargo clean

run: $(TARGET)
	$(RUN_COMMAND)

ifeq (,$(filter clean,$(MAKECMDGOALS)))
-include $(DFILES)
endif
