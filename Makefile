all:
.PHONY: up

up:
	export DOCKER_BUILDKIT=1
	export COMPOSE_DOCKER_CLI_BUILD=1
	docker compose up -d --build
	$(MAKE) -C ./proxy