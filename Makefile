.PHONY: dev build test lint

dev:
	docker compose up

build:
	docker compose build

test:
	$(MAKE) -C backend test
	$(MAKE) -C frontend test

lint:
	$(MAKE) -C backend lint
