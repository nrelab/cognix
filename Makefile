.DEFAULT_GOAL := help

DOCKER ?= docker
DOCKER_CONTEXT ?= .
DOCKERFILE ?= docker/agent-dev/Dockerfile
DOCKER_IMAGE ?= cognix-dev
DOCKER_TAG ?= latest
DOCKER_REGISTRY ?=
DOCKER_BUILD_ARGS ?=
DOCKER_REF = $(if $(DOCKER_REGISTRY),$(DOCKER_REGISTRY)/,)$(DOCKER_IMAGE):$(DOCKER_TAG)

.PHONY: help build run bundle test test-unit test-doc fmt fmt-check clippy lint presubmit bootstrap check clean docker-build docker-push docker docker-agent docker-linux

help:
	@printf 'Available targets:\n'
	@printf '  build        Build the workspace\n'
	@printf '  run          Build and run Cognix\n'
	@printf '  bundle       Bundle the main app\n'
	@printf '  test         Run all tests (nextest, v2 completer, doc tests)\n'
	@printf '  test-unit    Run workspace tests with nextest\n'
	@printf '  test-doc     Run doc tests\n'
	@printf '  fmt          Format Rust code\n'
	@printf '  fmt-check    Check Rust formatting\n'
	@printf '  clippy       Run clippy across the workspace\n'
	@printf '  lint         Run clippy, clang-format, and wgslfmt checks\n'
	@printf '  presubmit    Run all presubmit checks\n'
	@printf '  bootstrap    Run platform-specific setup\n'
	@printf '  check        Run formatting and clippy checks\n'
	@printf '  clean        Remove build artifacts\n'
	@printf '  docker-build Build a dev image (override DOCKERFILE/DOCKER_IMAGE)\n'
	@printf '  docker-push  Push the built dev image\n'
	@printf '  docker       Build and push the dev image\n'
	@printf '  docker-agent Build and push docker/agent-dev\n'
	@printf '  docker-linux Build and push docker/linux-dev\n'
	@printf '\n'
	@printf 'Docker variables: DOCKER_CONTEXT, DOCKERFILE, DOCKER_IMAGE, DOCKER_TAG, DOCKER_REGISTRY, DOCKER_BUILD_ARGS\n'

build:
	cargo build --workspace

run:
	cargo run

bundle:
	cd app && cargo bundle --bin cognix

test: test-unit test-doc
	cargo nextest run -p cognix_completer --features v2

test-unit:
	cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2

test-doc:
	cargo test --doc

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

clippy:
	cargo clippy --workspace --exclude cognix_completer --all-targets --all-features --tests -- -D warnings
	cargo clippy -p cognix_completer --all-targets --tests -- -D warnings

lint: clippy
	./script/run-clang-format.py -r --extensions 'c,h,cpp,m' ./crates/cognixui/src/ ./app/src/
	find . -name "*.wgsl" -exec wgslfmt --check {} +

presubmit:
	./script/presubmit

bootstrap:
	./script/bootstrap

check: fmt-check clippy

clean:
	cargo clean

docker-build:
	$(DOCKER) build $(DOCKER_BUILD_ARGS) -f $(DOCKERFILE) -t $(DOCKER_REF) $(DOCKER_CONTEXT)

docker-push:
	$(DOCKER) push $(DOCKER_REF)

docker: docker-build docker-push

docker-agent: DOCKERFILE := docker/agent-dev/Dockerfile
docker-agent: DOCKER_IMAGE := cognix-agent-dev
docker-agent: docker

docker-linux: DOCKERFILE := docker/linux-dev/Dockerfile
docker-linux: DOCKER_IMAGE := cognix-linux-dev
docker-linux: docker
