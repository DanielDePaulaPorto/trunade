---
layout: ../layouts/Docs.astro
title: Documentação
---

# Documentação

## Instalação

Requisitos: Rust, Node com pnpm e um compilador C++. No macOS, `brew install gcc` instala o `g++-16`; o `g++` da Apple é o Clang e não tem `<bits/stdc++.h>`.

```bash
TrunadeCode/install.sh
trunade aula.md
```

O comando abre a aula no navegador. Edite o `.md` no seu editor; a página se atualiza ao salvar.

## Blocos de código

Blocos cercados com uma linguagem configurada (C++, C e Python por padrão) viram células executáveis. Os demais continuam texto.

````markdown
```cpp {flags="-O2" timeout=5}
#include <iostream>
int main() { int a, b; std::cin >> a >> b; std::cout << a + b << "\n"; }
```

```stdin
2 40
```
````

- **Atributos:** `flags` (flags extras do compilador) e `timeout` (segundos).
- **Entrada fixa:** um bloco `stdin` logo depois do código. Sem ele, o programa roda num terminal e você digita a entrada na hora.

## Recursos de aula

- **Fórmulas:** `$x^2$` no texto, `$$ … $$` ou um bloco `math` em destaque.
- **Diagramas:** bloco `mermaid`.
- **Perguntas:** bloco `quiz`; `- [x]` marca as certas.
- **Exercício em passos:** marcadores `<!-- step: Título -->` … `<!-- /steps -->`.
- **HTML:** `<div>`, `<img>`, `<iframe>`. Para um site mais largo que o texto: `<iframe src="…" data-width="1200" height="600">`.

## Atalhos

| Tecla | Ação |
|---|---|
| `T` | índice de seções |
| `Espaço` / `↓` | próxima seção |
| `Shift+Espaço` / `↑` | seção anterior |
| `+` `-` `0` | fonte |
| `D` | tema |
| `W` | largura do texto (estreita ou página inteira) |
| `O` | configurações |
| `Shift+Enter` | executar o bloco |
| `Esc` | sair do código ou do terminal |

## Configuração

`~/.config/trunade/config.toml` vale para todas as aulas; um `trunade.toml` ao lado do `.md` sobrescreve só o que declarar.

```toml
timeout_seconds = 10
editor = ["zed", "{file}:{line}"]

[languages.cpp]
aliases = ["c++", "cxx", "cc"]
source = "main.cpp"
compile = ["g++-16", "-std=c++23", "-Wall", "{flags}", "{source}", "-o", "{binary}"]
run = ["{binary}"]
```
