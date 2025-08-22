LAB ?= 
LAB_DIR := labs/$(LAB)
CONFIG ?= config.toml

COMPOSE = docker compose

up: down
	$(COMPOSE) up -d

down:
	$(COMPOSE) down -v

.PHONY: guard-config
guard-config:
	@if [ ! -f "$(CONFIG)" ]; then \
	  echo "ERROR: Root config '$(CONFIG)' not found."; \
	  echo "Create it (e.g., copy config.example.toml) or pass CONFIG=<path>."; \
	  exit 1; \
	fi

bootstrap: guard-config
	cargo run --manifest-path shared/Cargo.toml --bin bootstrap -- \
		--config $(CONFIG) --lab-config $(LAB_DIR)/config.toml

recv: guard-config
	cargo run --manifest-path shared/Cargo.toml --bin recv -- \
 		--config $(CONFIG) --lab-config $(LAB_DIR)/config.toml $(ARGS)

send: guard-config
	cargo run --manifest-path shared/Cargo.toml --bin send -- \
 	  --config $(CONFIG) --lab-config $(LAB_DIR)/config.toml --msg "$(MSG)"

purge: guard-config
	cargo run --manifest-path shared/Cargo.toml --bin purge -- \
 	  --config $(CONFIG) --lab-config $(LAB_DIR)/config.toml

teardown: guard-config
	cargo run --manifest-path shared/Cargo.toml --bin teardown -- \
 	  --config $(CONFIG) --lab-config $(LAB_DIR)/config.toml