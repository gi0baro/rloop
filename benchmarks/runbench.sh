#!/usr/bin/env bash

rm -rf ./benchmarks/.envs
rm -rf ./target/wheels
mkdir -p ./benchmarks/.envs

uv venv -p 3.10 ./benchmarks/.envs/.venv310
uv venv -p 3.11 ./benchmarks/.envs/.venv311
uv venv -p 3.12 ./benchmarks/.envs/.venv312
uv venv -p 3.13 ./benchmarks/.envs/.venv313
uv venv -p 3.14 ./benchmarks/.envs/.venv314

uv sync --group build

uv run maturin build --release --interpreter ./benchmarks/.envs/.venv310/bin/python
uv run maturin build --release --interpreter ./benchmarks/.envs/.venv311/bin/python
uv run maturin build --release --interpreter ./benchmarks/.envs/.venv312/bin/python
uv run maturin build --release --interpreter ./benchmarks/.envs/.venv313/bin/python
uv run maturin build --release --interpreter ./benchmarks/.envs/.venv314/bin/python

VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv310 uv pip install $(ls target/wheels/rloop-*-cp310-*.whl)
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv311 uv pip install $(ls target/wheels/rloop-*-cp311-*.whl)
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv312 uv pip install $(ls target/wheels/rloop-*-cp312-*.whl)
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv313 uv pip install $(ls target/wheels/rloop-*-cp313-*.whl)
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv314 uv pip install $(ls target/wheels/rloop-*-cp314-*.whl)

VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv310 uv pip install numpy uvloop
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv311 uv pip install numpy uvloop
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv312 uv pip install numpy uvloop
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv313 uv pip install numpy uvloop
VIRTUAL_ENV=$(pwd)/benchmarks/.envs/.venv314 uv pip install numpy uvloop zuvloop

cd ./benchmarks

# base bench
BENCHMARK_EXC_PREFIX=$(pwd)/.envs/.venv314/bin ./.envs/.venv314/bin/python benchmarks.py all
mv ./results/data.json ./results/base.json

# pyver (NO! deve runnare solo su rloop)
BENCHMARK_EXC_PREFIX=$(pwd)/.envs/.venv310/bin BENCHMARK_SELF_ONLY=t ./.envs/.venv310/bin/python benchmarks.py raw
mv ./results/data.json ./results/py310.json
BENCHMARK_EXC_PREFIX=$(pwd)/.envs/.venv311/bin BENCHMARK_SELF_ONLY=t ./.envs/.venv311/bin/python benchmarks.py raw
mv ./results/data.json ./results/py311.json
BENCHMARK_EXC_PREFIX=$(pwd)/.envs/.venv312/bin BENCHMARK_SELF_ONLY=t ./.envs/.venv312/bin/python benchmarks.py raw
mv ./results/data.json ./results/py312.json
BENCHMARK_EXC_PREFIX=$(pwd)/.envs/.venv313/bin BENCHMARK_SELF_ONLY=t ./.envs/.venv313/bin/python benchmarks.py raw
mv ./results/data.json ./results/py313.json
BENCHMARK_EXC_PREFIX=$(pwd)/.envs/.venv314/bin BENCHMARK_SELF_ONLY=t ./.envs/.venv314/bin/python benchmarks.py raw
mv ./results/data.json ./results/py314.json
