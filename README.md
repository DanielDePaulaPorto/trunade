<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="site/public/brand/trunade-logo-escuro.svg">
    <img src="site/public/brand/trunade-logo.svg" alt="Trunade!" height="64">
  </picture>
</h1>

Aulas de programação escritas em Markdown, com blocos de código que compilam e rodam no navegador.

```bash
trunade aula.md
```

Documentação completa (sintaxe, exemplos, recursos de aula): [trunade.org](https://www.trunade.org) ou `site/src/pages/docs/`.

O comando abre a aula no navegador. Cada bloco ` ```cpp ` vira uma caixa editável com ▶; a saída aparece logo abaixo, num terminal. Você edita o `.md` no seu editor (Zed, VS Code, Vim…) e a página se atualiza ao salvar.

## Por que não o Jupyter?

O Jupyter é ótimo para análise de dados, onde cada célula continua o estado da anterior e o resultado (tabelas, gráficos) fica salvo no próprio notebook. Uma aula de programação em C++ pede outra coisa.

| | Trunade! | Jupyter |
|---|---|---|
| **Arquivo da aula** | Markdown puro. Diffs legíveis no git, abre em qualquer editor, não guarda saída nem metadados. | `.ipynb` em JSON, com saídas e metadados misturados ao texto. Conflitos de merge difíceis. |
| **O que é um bloco** | Um programa completo, com `main()`, compilado como o aluno compila na prova ou na maratona. | Uma célula que continua o estado das anteriores. Para C++ (xeus-cling), código interpretado, sem `main` e com semântica própria. |
| **Compilador** | O real (`g++-16` por padrão, configurável), com as flags que você escolher. Erros na íntegra, coloridos. | O do kernel. |
| **Erros de execução** | Código de saída, sinais (`Segmentation fault`), tempo limite e limite de saída aparecem na tela. | Dependem do kernel; um laço infinito trava a célula. |
| **Entrada padrão** | Interativa num terminal real, ou fixa num bloco ` ```stdin ` versionado junto com a aula. | `input()` limitado; em C++ geralmente não há `cin` interativo. |
| **Projeção em sala** | Tema claro, fonte ajustável, navegação por seções com espaço e índice com `T`, nada de barras de ferramentas. | Interface de notebook, feita para editar. |
| **Instalação** | Um binário, funciona offline. | Python, Jupyter e um kernel de C++. |

Use o Jupyter quando o estado entre células for o ponto (exploração de dados, gráficos). Use o Trunade! quando cada exemplo for um programa.

## Instalação

Requisitos: Rust, Node com pnpm e um compilador C++ (no macOS, `brew install gcc` instala o `g++-16`; o `g++` da Apple é o Clang e não tem `<bits/stdc++.h>`).

```bash
./install.sh
```

O script gera a interface web, embute no binário e instala `trunade` em `~/.cargo/bin`.

## Escrevendo uma aula

Blocos cercados com uma linguagem configurada viram células executáveis. Os demais continuam texto.

````markdown
```cpp {flags="-O2" timeout=5}
#include <iostream>
int main() { int a, b; std::cin >> a >> b; std::cout << a + b << "\n"; }
```

```stdin
2 40
```
````

- **Atributos** no fence: `flags` (flags extras do compilador) e `timeout` (segundos).
- **Entrada fixa:** um bloco ` ```stdin ` logo depois do código. Sem ele, o programa roda num terminal e você digita a entrada na hora (sem tempo limite; use ■ para parar).
- **Fórmulas:** `$x^2$` no texto, `$$ … $$` ou ` ```math ` em destaque (KaTeX).
- **Diagramas:** ` ```mermaid `.
- **Perguntas:** a primeira parte é o enunciado; `- [x]` marca as certas (mais de uma vira múltipla escolha).

  ````markdown
  ```quiz
  Qual a complexidade da busca binária?
  - [ ] O(n)
  - [x] O(log n)
  ```
  ````

- **Exercício em passos:** os blocos entre os marcadores viram um passo a passo (enunciado → dica → solução). O código dentro dos passos continua executável.

  ~~~markdown
  <!-- step: Enunciado -->
  Leia dois inteiros e imprima a soma.

  <!-- step: Dica -->
  Use `std::cin >> a >> b`.

  <!-- step: Solução -->
  ```cpp
  #include <iostream>
  int main() { int a, b; std::cin >> a >> b; std::cout << a + b << "\n"; }
  ```
  <!-- /steps -->
  ~~~

- **HTML** é aceito: `<div>`, `<img>`, `<iframe>`. Imagens e páginas ao lado do `.md` são servidas pelo próprio Trunade!. Iframes de páginas locais se ajustam sozinhos à largura do texto e à altura do conteúdo. Para um site externo mais largo que o texto, informe a largura que ele precisa e o Trunade! reduz para caber: `<iframe src="https://…" data-width="1200" height="600"></iframe>`.

## Na aula

| Tecla | Ação |
|---|---|
| `T` | índice de seções (→ abre as subseções) |
| `Espaço` / `↓` / `Page Down` | próxima seção, com rolagem suave |
| `Shift+Espaço` / `↑` / `Page Up` | seção anterior |
| `+` `-` `0` | aumentar, diminuir, restaurar a fonte |
| `D` | alternar tema |
| `⌘P` / `Ctrl+P` | exportar PDF (também no ☰) |
| `W` | largura do texto: estreita ou página inteira |
| `F` | tela cheia (`Esc` ou `F` sai) |
| `O` | configurações (também no ☰) |
| `Shift+Enter` | executar o bloco, com o cursor no código |
| `Esc` | sair do código ou do terminal |

No painel ☰ também dá para trocar as fontes (títulos, texto e código), a largura do texto, o idioma (português/inglês) e as cores do código (seguir o tema, Monokai claro ou escuro).

O tamanho de fonte do Trunade! muda só o conteúdo da página. Todas as escolhas do painel ficam em `~/.config/trunade/preferences.toml` e valem para qualquer aula. O zoom do navegador (`⌘ +`) continua funcionando e se multiplica com ele.

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

C++, C e Python vêm configurados.
