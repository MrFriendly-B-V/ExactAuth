.PHONY: run-proxy
all: run-proxy

run-proxy:
	RUST_LOG=simpleproxy=INFO,actix_server=INFO ./simpleproxy --config ./config.toml

setup-hosts:
	sudo bash -c 'echo "127.0.0.1 mrf.local" >> /etc/hosts'
