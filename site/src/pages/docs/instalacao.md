---
layout: ../../layouts/Docs.astro
title: Instalação
description: Como instalar o Trunade! no macOS e no Linux.
---

# Instalação

O Trunade! roda no macOS e no Linux.

## Requisitos

- **Rust** (para compilar o binário).
- **Node com pnpm** (para gerar a interface, que vai embutida no binário).
- **Um compilador C++.** No macOS, instale o GCC pelo Homebrew:

```bash
brew install gcc
```

> O `g++` que vem com o macOS é o Apple Clang e não tem `<bits/stdc++.h>`. Por isso o padrão do Trunade! é `g++-16`. Dá para trocar na [configuração](/docs/configuracao/).

## Instalar

Na pasta do repositório:

```bash
./install.sh
```

O script gera a interface web, embute no binário e instala `trunade` em `~/.cargo/bin`.

## Usar

```bash
trunade aula.md
trunade aula.md --port 5000
trunade aula.md --no-open
trunade aula.md --config outra.toml
```

Se a porta 4747 estiver ocupada, o Trunade! escolhe outra livre e mostra o endereço no terminal.

## Atualizar

Rode o `install.sh` de novo e reinicie o `trunade` que estiver aberto.
