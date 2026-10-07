---
layout: ../../layouts/Docs.astro
title: Visão geral
description: O que é o Trunade! e como ele funciona.
---

# Visão geral

Trunade! mostra uma aula escrita em Markdown no navegador. Cada bloco de código vira uma caixa editável com ▶: ao clicar, o programa é compilado com o compilador da sua máquina e a saída aparece num terminal logo abaixo.

```bash
trunade aula.md
```

O comando serve a aula em `http://localhost:4747` e abre o navegador. Você continua editando o `.md` no seu editor; ao salvar, a página atualiza.

## Como funciona

- **Um binário.** O `trunade` é um servidor local escrito em Rust, com a interface embutida. Escuta só na sua máquina e funciona sem internet.
- **Cada bloco é um programa.** Não há estado entre blocos: cada um tem o seu `main()` e é compilado do zero, como o aluno faz na prova.
- **A aula não muda.** Editar o código no navegador não grava no arquivo. Recarregou, voltou ao que está no `.md`.
- **Compilador de verdade.** C++, C e Python vêm configurados. Outras linguagens entram pelo arquivo de configuração.

## Para quem é

Para quem dá aula de programação, algoritmos e maratona e quer rodar exemplos no telão sem alternar entre slides, editor e terminal.

Se o estado entre células é o ponto da aula (análise de dados, gráficos), o Jupyter continua sendo a ferramenta certa.
