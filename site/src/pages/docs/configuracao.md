---
layout: ../../layouts/Docs.astro
title: Configuração
description: Compiladores, linguagens, tempos limite e editor.
---

# Configuração

Dois arquivos, lidos nesta ordem:

1. `~/.config/trunade/config.toml`, para todas as aulas.
2. `trunade.toml` ao lado do `.md`, só para aquela pasta.

Cada um sobrescreve só o que declara. Com `--config arquivo.toml`, só esse arquivo é lido.

## Exemplo

```toml
timeout_seconds = 10
compile_timeout_seconds = 60
output_limit_bytes = 1048576
editor = ["zed", "{file}:{line}"]

[languages.cpp]
aliases = ["c++", "cxx", "cc"]
source = "main.cpp"
compile = ["g++-16", "-std=c++23", "-Wall", "-Wextra", "-fdiagnostics-color=always", "{flags}", "{source}", "-o", "{binary}"]
run = ["{binary}"]
```

## Campos

| Campo | O que faz |
|---|---|
| `timeout_seconds` | tempo limite de execução com `stdin` fixo |
| `compile_timeout_seconds` | tempo limite da compilação |
| `output_limit_bytes` | limite de saída por execução |
| `editor` | comando para abrir o `.md` na linha do bloco |

## Linguagens

Cada `[languages.nome]` substitui a linguagem inteira.

| Campo | O que faz |
|---|---|
| `aliases` | outros nomes aceitos no fence |
| `source` | nome do arquivo-fonte |
| `compile` | comando de compilação (opcional; Python não tem) |
| `run` | comando de execução |

Nos comandos, `{source}` e `{binary}` viram os caminhos temporários e `{flags}` vira as flags do bloco, em vários argumentos.

### Exemplo: Rust

```toml
[languages.rust]
aliases = ["rs"]
source = "main.rs"
compile = ["rustc", "-O", "{flags}", "{source}", "-o", "{binary}"]
run = ["{binary}"]
```
