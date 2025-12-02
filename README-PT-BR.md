[![English](https://img.shields.io/badge/English-Inactive-lightgrey?style=flat-square)](README.md) [![Русский](https://img.shields.io/badge/Русский-Inactive-lightgrey?style=flat-square)](README-RU.md) [![Português](https://img.shields.io/badge/Português-Active-success?style=flat-square)](README-PT-BR.md)

<!-- Logo do Projeto -->
<p align="center">
  <img src=".github/assets/logo.svg" alt="Logo Oxide Lab" width="512" />
</p>

> **Chat de IA privado, poderoso e fácil de usar diretamente no seu computador**

![Oxide Lab](https://img.shields.io/badge/Status-Active-brightgreen) ![Platform](https://img.shields.io/badge/Platform-Windows-blue) ![License](https://img.shields.io/badge/License-MIT-yellow) ![Legal](https://img.shields.io/badge/Legal-Compliant-green)

![GitHub Stars](https://img.shields.io/github/stars/FerrisMind/Oxide-Lab?style=social) [![Awesome Tauri](https://awesome.re/mentioned-badge.svg)](https://github.com/tauri-apps/awesome-tauri) [![Awesome Svelte](https://awesome.re/mentioned-badge.svg)](https://github.com/TheComputerM/awesome-svelte)

---

## 📚 Índice

- [O que é isso?](#-o-que-é-isso)
- [Para quem é este aplicativo?](#-para-quem-é-este-aplicativo)
- [Recursos Principais](#-recursos-principais)
- [Reconhecimento](#️-reconhecimento)
- [Instalação e Configuração](#️-instalação-e-configuração)
- [Como Começar a Usar](#-como-começar-a-usar)
- [Recursos da Interface](#-recursos-da-interface)
- [Privacidade e Segurança](#-privacidade-e-segurança)
- [Dicas e Recomendações](#-dicas-e-recomendações)
- [Requisitos do Sistema e Limitações](#-requisitos-do-sistema-e-limitações)
- [Apoiar o Projeto](#-apoiar-o-projeto)
- [Agradecimentos](#-agradecimentos)

---

## ✨ O que é isso?

**Oxide Lab** é um aplicativo desktop moderno para comunicação com modelos de IA que funciona completamente localmente no seu computador. Sem assinaturas, sem envio de dados para a internet — apenas você e seu assistente de IA pessoal.

### 🎯 Para quem é este aplicativo?

- **Entusiastas de IA** — querem experimentar modelos localmente
- **Privacidade importa** — seus dados ficam apenas com você
- **Pesquisadores** — precisam de controle sobre parâmetros de geração
- **Mentes criativas** — usam IA para escrita, brainstorming e inspiração

---

## 🚀 Recursos Principais

### 💬 **Interface de Chat Inteligente**

- Design moderno e intuitivo
- Respostas em streaming em tempo real
- Suporte para formatação de texto e código

### 🧠 **Modo de Pensamento**

- Ative o recurso **"Pensamento"** e observe a IA pensar
- Veja o processo de análise antes da resposta final
- Soluções de maior qualidade e mais pensadas para tarefas complexas

### ⚙️ **Configurações Flexíveis**

- **Temperatura** — controle a criatividade da resposta
- **Top-K, Top-P, Min-P** — ajuste fino do estilo de geração
- **Penalidade de Repetição** — evite repetições
- **Comprimento do Contexto** — depende do modelo e dos recursos do dispositivo

### 🔧 **Configuração Fácil**

- Suporte para modelos Qwen3 locais no formato GGUF (outros modelos — em planejamento)
- Gerenciamento inteligente de memória

---

## 🎖️ Reconhecimento

Oxide Lab foi reconhecido pela comunidade por sua qualidade e inovação:

- ⭐ **100+ estrelas no GitHub** nos primeiros 3-4 meses de desenvolvimento solo
- 🏆 **Destaque em [Awesome Tauri](https://github.com/tauri-apps/awesome-tauri)** — lista curada de aplicativos Tauri de qualidade
- 🏆 **Destaque em [Awesome Svelte](https://github.com/TheComputerM/awesome-svelte)** — lista curada de projetos Svelte de qualidade

---

## 🛠️ Instalação e Configuração

### Pré-requisitos

Antes de instalar o Oxide Lab, certifique-se de ter o seguinte instalado:

- **Node.js** (versão 18 ou superior) - [Download](https://nodejs.org/)
- **Rust** (última versão estável) - [Instalar](https://rustup.rs/)
- **Git** - [Download](https://git-scm.com/)

#### Para Aceleração GPU (Opcional mas Recomendado)

- **CUDA 12.0+** para GPUs NVIDIA (Windows/Linux)

### Passos de Instalação

1. **Clone o repositório:**

   ```bash
   git clone https://github.com/FerrisMind/Oxide-Lab.git
   cd Oxide-Lab
   ```

2. **Instale as dependências:**

   ```bash
   npm install
   ```

3. **Execute em modo de desenvolvimento:**

   ```bash
   # Para modo apenas CPU
   npm run tauri:dev:cpu

   # Para modo GPU CUDA (se CUDA estiver disponível)
   npm run tauri:dev:cuda
   ```

4. **Compile para produção:**

   ```bash
   # Compilação apenas CPU
   npm run tauri:build:cpu

   # Compilação CUDA
   npm run tauri:build:cuda
   ```

### Requisitos do Sistema

- **SO:** Windows 10/11, Linux, macOS
- **RAM:** Mínimo 4GB, Recomendado 8GB+
- **Armazenamento:** 500MB para aplicativo + tamanho do modelo
- **GPU:** Opcional, mas recomendado para melhor desempenho

### Solução de Problemas

- Se encontrar problemas de compilação, certifique-se de que Rust e Node.js estão instalados corretamente
- Para suporte GPU, verifique a instalação do CUDA
- Verifique a página [Issues](https://github.com/FerrisMind/Oxide-Lab/issues) para problemas comuns

---

## 📖 Como Começar a Usar

### 1️⃣ **Obtenha o Modelo**

Baixe um modelo no formato `.gguf` e o arquivo `tokenizer.json`:

- **Modelos recomendados:** Qwen3 4B (e outras variantes Qwen3 em GGUF)
- **Onde baixar:** [Hugging Face](https://huggingface.co/collections/Qwen/qwen3-67dd247413f0e2e4f653967f), repositórios oficiais de modelos

### 2️⃣ **Carregue no Aplicativo**

1. Abra o Oxide Lab
2. Clique em **"Selecionar Arquivo do Modelo"** e especifique o caminho para o arquivo `.gguf`
3. Opcionalmente configure os parâmetros de inferência
4. Clique em **"Carregar"**

### 3️⃣ **Comece a Conversar**

- Digite sua pergunta ou solicitação
- Ative **"Pensamento"** para respostas mais profundas
- Ajuste os parâmetros de geração ao seu gosto
- Aproveite a conversa com sua IA pessoal!

---

## 🎨 Recursos da Interface

### 📊 **Indicadores Informativos**

- Progresso de carregamento do modelo com estágios detalhados
- Indicadores de status de geração
- Exibição visual do pensamento da IA

<p align="center">
  <img src=".github/assets/screenshots/chat-dark.png" alt="Interface de Chat Oxide Lab (Escuro)" width="720" />
</p>

### ⚡ **Ações Rápidas**

- Cancele o carregamento do modelo com um clique
- Pare a geração a qualquer momento
- Mudanças rápidas de parâmetros sem recarregar

---

## 🛡️ Privacidade e Segurança

### 🔒 **100% Local**

- Todos os cálculos acontecem no seu computador
- Sem solicitações externas ou envio de dados
- Controle total sobre suas informações

### 💾 **Gerenciamento de Dados**

- Conversas armazenadas apenas na sessão do aplicativo
- Modelos permanecem no seu disco
- Sem coleta oculta de dados

---

## 💡 Dicas e Recomendações

### 🎯 **Para melhores resultados:**

- Use o modo de pensamento para tarefas complexas
- O aplicativo já possui as melhores configurações integradas com base nas recomendações do fabricante do modelo Qwen3. Basta ativar e usar!
- O aplicativo também suporta alteração das configurações padrão. Experimente com temperatura: 0.7-1.0 para criatividade, 0.1-0.3 para precisão
- Aumente o contexto para trabalhar com documentos longos

### ⚡ **Otimização de desempenho:**

- Suporta CPU e GPU (CUDA)

### 🎨 **Uso criativo:**

- Ative o pensamento para análise de texto e resolução de problemas
- Experimente alta temperatura para escrita criativa
- Use contexto longo para trabalhar com documentos grandes

---

## 🖥️ Requisitos do Sistema e Limitações

### Plataformas Suportadas

- Windows 10/11 — suporte completo
- Linux e macOS — em fase de planejamento (ainda não suportados)

### Modelos

- Suportados: Qwen3 no formato GGUF (mono-arquitetura)
- Importante: compatibilidade com outros modelos ainda não é garantida

### Requisitos Mínimos de Hardware

Os menores modelos Qwen3 (0.6B e 1.7B) funcionam com velocidade e qualidade aceitáveis mesmo em dispositivos com CPU de 2 núcleos e 4 GB de RAM. O modelo 4B também funciona neste aplicativo com tais dispositivos, mas o desempenho é muitas vezes menor e requer mais memória, o que é difícil de alcançar, por exemplo, com LM Studio sem perda significativa de qualidade.

### Contexto e Desempenho

- O comprimento efetivo do contexto depende de: modelo selecionado, RAM disponível
- O comprimento do contexto praticamente alcançável pode ser menor do que o teoricamente declarado pelo modelo
- Quanto maior o contexto, maiores os requisitos de memória e menor a velocidade de geração

---

## 🌟 Apoiar o Projeto

Se o Oxide Lab foi útil para você:

- ⭐ Dê uma estrela ao projeto
- 🐛 Reporte bugs
- 💡 Sugira novos recursos
- 🤝 Compartilhe com amigos

---

## 🙏 Agradecimentos

Oxide Lab é construído com a ajuda de tecnologias incríveis de código aberto:

- **[Rust](https://www.rust-lang.org/)** - Linguagem de programação de sistemas que garante segurança de memória e desempenho
- **[Tauri](https://tauri.app/)** - Framework para construir aplicativos desktop rápidos e seguros
- **[Candle](https://github.com/huggingface/candle)** - Framework ML minimalista para Rust
- **[Phosphor Icons](https://phosphoricons.com/)** - Conjunto de ícones bonito e consistente

### Tradução

- **Tradução para português brasileiro:** Talita Maia Sousa

---

> **Feito com ❤️ para a comunidade de entusiastas de IA**  
> _Liberdade, privacidade e controle sobre inteligência artificial_


