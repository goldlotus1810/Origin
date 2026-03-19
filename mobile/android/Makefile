# HomeOS — Android ARM64 Makefile
# Usage:
#   make              — build for Termux (on-device)
#   make ndk          — build via NDK (cross-compile)
#   make deploy       — push to device via adb
#   make run          — run on device via adb

ROOT     = ../..
VM_SRC   = $(ROOT)/vm/arm64/vm_arm64.S
STDLIB   = $(ROOT)/stdlib
OUTPUT   = origin.olang

.PHONY: all ndk deploy run clean

all:
	bash build.sh $(OUTPUT)

ndk:
	@test -n "$(ANDROID_NDK_HOME)" || (echo "Set ANDROID_NDK_HOME first" && exit 1)
	bash build.sh $(OUTPUT)

deploy: $(OUTPUT)
	adb push $(OUTPUT) /data/local/tmp/origin.olang
	adb shell chmod +x /data/local/tmp/origin.olang

run:
	adb shell /data/local/tmp/origin.olang

clean:
	rm -f $(OUTPUT) /tmp/vm_arm64.o /tmp/vm_arm64
