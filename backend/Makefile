.PHONY: build run test lint tidy

build:
	go build -o bin/server ./cmd/server

run:
	go run ./cmd/server

test:
	go test -race ./...

lint:
	golangci-lint run ./...

tidy:
	go mod tidy
