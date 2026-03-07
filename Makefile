.PHONY: build run dev clean test

# One-liner install (giống Docker)
# curl -fsSL https://raw.githubusercontent.com/goldlotus1810/Origin/main/install.sh | bash

build:
	go build -o homeos ./cmd/homeos/

run: build
	HOMEOS_STATIC=web/static ./homeos

dev:
	HOMEOS_STATIC=web/static go run ./cmd/homeos/

clean:
	rm -f homeos

test:
	go test ./...

lint:
	go vet ./...
