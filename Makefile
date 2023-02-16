TARGET := condux
BINARY := $(TARGET)

BUILDDIR := build
SRCDIR := src

CFILES := \
	$(SRCDIR)/assets.c \
	$(SRCDIR)/collision.c \
	$(SRCDIR)/input.c \
	$(SRCDIR)/linalg.c \
	$(SRCDIR)/main.c \
	$(SRCDIR)/render.c \
	$(SRCDIR)/spline.c \
	$(SRCDIR)/state.c \
	$(SRCDIR)/timing.c \
	$(SRCDIR)/vehicle.c

CFLAGS := -Wall -Oz -flto -Iinclude -Ibuild

CC := gcc

LDFLAGS := -lm

PLATFORM ?= sdl

ERR =

ifeq ($(PLATFORM), sdl)
	LDFLAGS += -lSDL2 -lGL
	CFILES += $(SRCDIR)/platform/sdl.c
	RUN_COMMAND := ./$(TARGET)
else ifeq ($(PLATFORM), web)
	# TODO more portable
	CC := clang
	CFLAGS += \
		--target=wasm32-unknown-wasi \
		--sysroot=/usr/share/wasi-sysroot \
		-DCONDUX_WEB=1 \
		-DNDEBUG
	LDFLAGS += \
		-mexec-model=reactor \
		-Wl,--no-entry
	BINARY := web/index.wasm
	TARGET := $(BINARY)
	RUN_COMMAND := python scripts/run_web.py
else ifeq ($(PLATFORM), wii)
	CC := $(DEVKITPPC)/bin/powerpc-eabi-gcc
	CFLAGS += \
		-DGEKKO \
		-mrvl \
		-mcpu=750 \
		-meabi \
		-mhard-float \
		-I$(DEVKITPRO)/libogc/include \
		-L$(DEVKITPRO)/libogc/lib/wii
	LDFLAGS += -lwiiuse -lbte -logc -lm
	BINARY := $(BINARY).elf
	TARGET := $(TARGET).dol
	CFILES += $(SRCDIR)/platform/gx.c
	RUN_COMMAND := dolphin-emu $(TARGET)
else ifeq ($(PLATFORM), 3ds)
	CC := $(DEVKITARM)/bin/arm-none-eabi-gcc
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
	BINARY := $(BINARY).elf
	TARGET := $(TARGET).3dsx
	CFILES += $(SRCDIR)/platform/ctr.c
	RUN_COMMAND := citra-qt $(TARGET)
else
	ERR = $(error unknown platform: $(PLATFORM))
endif

OFILES := $(CFILES:$(SRCDIR)/%.c=$(BUILDDIR)/%.o)
DFILES := $(CFILES:$(SRCDIR)/%.c=$(BUILDDIR)/%.d)

$(ERR)

.PHONY: all clean run

all: $(TARGET)

%.3dsx: %.elf
	3dsxtool $< $@

%.dol: %.elf
	elf2dol $< $@

$(BUILDDIR)/assets.o: $(BUILDDIR)/bundle.h
$(BUILDDIR)/assets.d: $(BUILDDIR)/bundle.h

$(BUILDDIR)/bundle.h: $(shell find assets)
	mkdir -p $(dir $@)
	python scripts/asset_bundler.py

$(BINARY): $(OFILES)
	$(CC) $(CFLAGS) -o $@ $(OFILES) $(LDFLAGS)

$(BUILDDIR)/%.d: $(SRCDIR)/%.c
	mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -MM -MT $(<:$(SRCDIR)/%.c=$(BUILDDIR)/%.o) $< -MF $@

$(BUILDDIR)/%.o: $(SRCDIR)/%.c
	mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -rf $(BUILDDIR) $(TARGET) $(BINARY)

run: $(TARGET)
	$(RUN_COMMAND)

ifeq (,$(filter clean,$(MAKECMDGOALS)))
-include $(DFILES)
endif
