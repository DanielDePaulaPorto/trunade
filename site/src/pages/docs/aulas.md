---
layout: ../../layouts/Docs.astro
title: Escrevendo aulas
description: Blocos de código, atributos e entrada padrão.
---

# Escrevendo aulas

A aula é Markdown comum. Blocos cercados com uma linguagem configurada viram programas; os demais continuam texto.

## Blocos de código

````markdown
```cpp
#include <iostream>
int main() { std::cout << "Olá, turma!\n"; }
```
````

Linguagens e apelidos que vêm configurados:

| Linguagem | Fence |
|---|---|
| C++ | `cpp`, `c++`, `cxx`, `cc` |
| C | `c` |
| Python | `python`, `py`, `python3` |

## Atributos

Entre chaves, depois da linguagem:

````markdown
```cpp {flags="-O2 -g" timeout=5}
```
````

- `flags`: flags extras para o compilador, além das da configuração.
- `timeout`: tempo limite em segundos para este bloco.

## Entrada padrão

Há dois jeitos de dar entrada a um programa.

### Fixa, junto com a aula

Um bloco `stdin` logo depois do código (só linhas em branco entre eles):

````markdown
```cpp
#include <iostream>
int main() { int a, b; std::cin >> a >> b; std::cout << a + b << "\n"; }
```

```stdin
2 40
```
````

O programa roda com essa entrada, com tempo limite, e stdout e stderr aparecem separados.

### Interativa, na hora

Sem bloco `stdin`, o programa roda num terminal de verdade: a turma digita a entrada no próprio terminal da página. Nesse modo não há tempo limite (o programa pode estar esperando alguém digitar). Use ■ para parar.

## O que aparece embaixo

- **Erro de compilação:** a saída do compilador inteira, colorida.
- **Código de saída diferente de zero**, **sinais** (`Segmentation fault`), **tempo limite** e **limite de saída** aparecem depois da saída do programa.
- O tempo de compilação e o de execução ficam na barra do bloco.

## Editando durante a aula

O código de cada bloco é editável no navegador. `Shift+Enter` roda o bloco com o cursor dentro dele. A edição não grava no `.md`: quando o arquivo muda, só os blocos alterados perdem a edição.
