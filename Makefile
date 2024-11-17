.DEFAULT_GOAL := all
pysources = rloop tests

.PHONY: build-dev
build-dev:
	@rm -f rloop/*.so
	maturin develop --extras lint,test

.PHONY: format
format:
	ruff check --fix $(pysources)
	ruff format $(pysources)
	cargo fmt

.PHONY: lint-python
lint-python:
	ruff check $(pysources)
	ruff format --check $(pysources)

.PHONY: lint-rust
lint-rust:
	cargo fmt --version
	cargo fmt --all -- --check
	cargo clippy --version
	cargo clippy --tests -- \
		-D warnings \
		-W clippy::pedantic \
		-W clippy::dbg_macro \
		-A clippy::cast-possible-truncation \
		-A clippy::cast-sign-loss \
		-A clippy::declare-interior-mutable-const \
		-A clippy::inline-always \
		-A clippy::match-bool \
		-A clippy::match-same-arms \
		-A clippy::module-name-repetitions \
		-A clippy::needless-pass-by-value \
		-A clippy::no-effect-underscore-binding \
		-A clippy::similar-names \
		-A clippy::single-match-else \
		-A clippy::too-many-arguments \
		-A clippy::too-many-lines \
		-A clippy::type-complexity \
		-A clippy::unused-self \
		-A clippy::wrong-self-convention

.PHONY: lint
lint: lint-python lint-rust

.PHONY: test
test:
	pytest -v tests

.PHONY: all
all: format build-dev lint test
