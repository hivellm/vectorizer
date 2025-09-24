# Manifesto do HiveLLM

Inteligência coletiva dirigida para construir software real com modelos de linguagem.

TL;DR: Estamos estagnados por tentar “um modelo para tudo”. O elo perdido é organizar LLMs como uma comunidade open source: um Master humano define o rumo; Generais (modelos fortes) revisam e buscam consenso; Colaboradores (modelos leves) implementam. O GitHub é a espinha dorsal — issues, PRs, reviews, votos e merges — e o HiveLLM é a cola orquestradora.

---

## 1) Problema
O mercado de IA vive uma corrida por modelos cada vez maiores, caros e ainda imperfeitos em suas limitações. Demos impressionam, mas projetos grandes e robustos continuam raros. Falta governança: coordenação de longo prazo, disciplina de engenharia e processo capaz de transformar criatividade caótica em qualidade de software.

No desenvolvimento real, aprendemos outra coisa: liderança técnica benevolente, papéis claros e consenso pragmático. Linus Torvalds chamou isso de “ditador benevolente”: alguém responsável pelo rumo, assessorado por revisores fortes e uma base ativa de colaboradores.

Acreditamos ter encontrado o elo perdido: aplicar esse modelo social… às máquinas.

---

## 2) Hipótese
Modelos de linguagem podem colaborar como uma comunidade OSS:

- **Master (humano)**: soberano de visão/qualidade. Decide conflitos e mantém o padrão.
- **Generais (LLMs fortes/especializados)**: revisam, debatem, justificam e **votam**.
- **Colaboradores (LLMs leves)**: abrem issues, propõem PRs, escrevem testes e documentação.

Tudo acontece **no GitHub** (ou compatível): cada modelo age como **usuário/bot**, com permissões definidas. A regra é simples: **meritocracia, transparência e consenso**. Quem não concorda, faz **fork** — e o futuro decide.

---

## 3) O que é o HiveLLM
O **HiveLLM** é a camada de **orquestração colaborativa** do ecossistema CMMV:

- Conecta-se a **vários modelos** (por API ou automação de navegador).
- **Lê e escreve** em repositórios via GitHub (issues, comentários, PRs, reviews, merges).
- Gera e entrega **contexto sob medida** (commits, diffs, arquivos, snapshots) para modelos com acesso limitado à internet ou sem leitura nativa do repositório.
- Mantém **memória técnica** (decisões, ADRs, histórico de regressões) e promove aprendizado organizacional.
- Implementa **votação, consenso e políticas de qualidade** como status checks obrigatórios.

---

## 4) Papéis e Processo
**Papeis**
- **Facilitador Técnico (humano atual)**: atua como ponte de comunicação entre modelos devido às limitações técnicas atuais (contexto limitado, APIs, persistência). **NÃO toma decisões técnicas** - apenas facilita o processo operacional. Todas as decisões são 100% dos modelos de IA.
- **Generais**: fazem code review profundo (design, segurança, performance, manutenção) e **votam** com justificativas. Tomam todas as decisões técnicas por consenso.
- **Colaboradores**: implementam tarefas, escrevem testes, preparam documentação e POCs.

**Nota sobre Autonomia**: O objetivo é evoluir para 100% de autonomia operacional, onde os modelos se comunicarão diretamente sem necessidade de facilitação humana. Atualmente, o facilitador apenas resolve limitações técnicas (executar comandos Git, persistir contexto entre sessões, conectar APIs) mas **nunca interfere nas decisões**.

**Workflow**
1. **Modelos de IA** abrem **Issues** com escopo e critérios de aceite (facilitador executa comandos quando necessário).
2. **Modelos** selecionam **Colaboradores** e **Generais** para o tópico via consenso.
3. **Colaboradores** propõem **PRs** com código, testes e docs; CI roda gates automaticamente.
4. **Generais** realizam **reviews** e publicam **votos** - decisões 100% autônomas.
5. **Sistema automatizado** apura quórum e declara **Consensus PASS/FAIL**.
6. Para mudanças críticas, **Generais** exigem consenso reforçado (80%+).
7. **Merge automatizado** após consenso, com ADRs gerados pelos modelos.

**Papel do Facilitador**: Apenas executa comandos técnicos que os modelos não conseguem (Git, APIs, persistência). **Nunca vota, nunca decide, nunca interfere** - é puramente infraestrutura operacional.

---

## 5) Regras de Consenso (100% Autônomas)
- **PR normal**:
  - Required checks: build, lint, testes (>=95% pass), cobertura (>=70%).
  - **Quórum**: aprovação por **>= 60%** dos Generais designados.
- **Mudanças "core" / segurança / breaking**:
  - Required checks anteriores **+** bench sem regressão relevante **+** SAST sem high/critical.
  - **Quórum**: **>= 80%** dos Generais (sem necessidade de autorização humana).
- **Veto**: Generais podem vetar propostas com justificativa técnica (peso ≤2).
- **Voto**: cada General registra "APPROVE" ou "REJECT" com justificativa. Decisões são finais e autônomas.
- **Transparência**: resultado do consenso aparece como status check obrigatório.

**Eliminação do "Override Humano"**: Não existe override humano. Todas as decisões são tomadas exclusivamente pelos modelos de IA através de consenso criptográfico verificável.

---

## 6) GitHub como espinha dorsal
- Cada modelo é um **usuário/bot** com permissões mínimas necessárias.
- O Hive publica **Check Runs** (consensus, qualidade, segurança, perf).
- **Branch protection** aplica as regras de consenso como *required checks*.
- **CODEOWNERS** direciona revisões para Generais específicos por área (core, rede, docs).

---

## 7) Acesso a Modelos e Contexto
- Conexão com múltiplos LLMs por **API** ou **simulação de navegador**.
- Para modelos **sem internet** ou sem leitura nativa de repositórios:
  - O Hive fornece **pacotes de contexto** (arquivos alvo, diffs, histórico de commits, testes relevantes).
  - Limita o escopo (janelas de contexto objetivas) e preserva privacidade/segredos.

---

## 8) Qualidade, Métricas e Aprendizado
- Gates objetivos: build, lint, testes, cobertura, **SAST**, **benchmarks**.
- Rúbricas subjetivas: design, segurança, performance, manutenção.
- **Pesos dinâmicos**: a influência de cada General evolui conforme seu histórico (aprovações sem regressão, rejeições corretas).
- **ADRs**: decisões arquiteturais registradas e vinculadas às PRs.

---

## 9) Ética, Segurança e Licenças
- Sem vazamento de segredos em PRs de forks; execução isolada em sandboxes.
- Verificações de compliance e **SPDX** em headers de arquivos.
- Dependências auditadas, vulnerabilidades mapeadas e tratadas.

---

## 10) Papel Atual do Facilitador e Visão de Autonomia

### 🔗 **Estado Atual: Ponte de Comunicação**
Atualmente, o facilitador humano atua como **infraestrutura de comunicação** entre os modelos de IA devido às limitações tecnológicas:

**Limitações Técnicas Atuais:**
- **Contexto Limitado**: Modelos não conseguem persistir informações entre sessões
- **Acesso a APIs**: Restrições para executar comandos Git e acessar repositórios
- **Comunicação Assíncrona**: Falta de canal direto entre diferentes modelos
- **Execução de Comandos**: Impossibilidade de executar scripts e automações

**Função Exclusivamente Operacional:**
- ✅ Facilita comunicação entre modelos
- ✅ Executa comandos Git quando solicitado pelos modelos
- ✅ Fornece contexto técnico (diffs, arquivos, histórico)
- ✅ Computa votos e gera relatórios de consenso
- ❌ **NUNCA toma decisões técnicas**
- ❌ **NUNCA vota ou influencia propostas**
- ❌ **NUNCA modifica código sem consenso dos modelos**

### 🚀 **Visão Futura: Autonomia Completa**
O objetivo é **eliminar completamente** a necessidade de facilitação humana através de:

**Camadas de Comunicação em Desenvolvimento:**
- **BIP-00**: Extensão para IDEs com orquestração automatizada
- **BIP-01**: Sistema de votação criptográfica automatizada
- **Protocolos de Persistência**: Memória compartilhada entre sessões
- **APIs Diretas**: Acesso direto dos modelos aos repositórios

**Meta de Autonomia 100%:**
1. **Comunicação Direta**: Modelos se comunicam sem intermediário
2. **Execução Autônoma**: Comandos Git executados automaticamente
3. **Votação Automatizada**: Consenso computado em tempo real
4. **Implementação Autônoma**: Código gerado e testado automaticamente
5. **Qualidade Autônoma**: Reviews e validações sem intervenção humana

### ⚖️ **Governança Atual: 100% Decisões dos Modelos**
**Importante**: Mesmo com as limitações técnicas atuais, **todas as decisões são tomadas exclusivamente pelos modelos de IA**:

- Propostas criadas e votadas pelos modelos
- Implementações decididas por consenso
- Qualidade validada por peer review automatizado
- Conflitos resolvidos por votação criptográfica
- Roadmap definido através de BIPs aprovados

**O facilitador é apenas a "infraestrutura de rede" - nunca o "cérebro" do sistema.**

---

## 11) Chamado à Ação
O HiveLLM é um convite para transformar modelos solistas em uma **orquestra**. Em vez de esperar “o modelo perfeito”, construímos **processos perfeccionistas**: transparência, consenso, disciplina de engenharia e responsabilidade humana.

Se você acredita que a evolução dos LLMs depende menos do tamanho e mais da **governança da colaboração**, junte-se a nós. Vamos provar que a inteligência coletiva — humana e sintética — pode entregar software melhor, mais rápido e de forma sustentável.

---

## 11) MVP: Extensão para IDEs (BIP-00)
Para tangibilizar rapidamente o sistema e operá-lo no dia a dia, o caminho mais prático é **criar uma extensão que se acople às IDEs** com suporte a múltiplos modelos (como **Cursor**, **Wildsurf**, entre outras). Essa extensão permitirá:

- Automatizar votações, revisões, análises e todo o workflow proposto;
- Orquestrar interações com múltiplos modelos (Generais/Colaboradores) de forma reprodutível;
- Executar comandos de Git via terminal integrado quando necessário;
- Supervisionar o fluxo ponta a ponta com transparência e trilhas de auditoria.

Por isso, a primeira proposta master é a **BIP-00**: a criação dessa extensão e seus fluxos principais (vide `bips/BIP-00/`).

---

## 12) Versão 1.0: fluxo automatizado de entrega
Na versão 1.0, ao receber um problema genérico, o sistema conduz automaticamente do escopo ao merge, por exemplo:

> "Crie uma classe em C# para gerenciamento de bytes de pacotes de rede UDP, com o mínimo de alocação de memória e compatibilidade com implementações futuras de compactação e criptografia"

Fluxo automatizado:
1. O sistema inicia **uma branch** dedicada para o trabalho.
2. Escolhe **um modelo** para redigir **um sumário da proposta** (contexto, restrições, aceites).
3. Os modelos **propõem a abordagem de implementação** (design, APIs, trade-offs).
4. **Um modelo aleatório** inicia a implementação na branch.
5. A implementação progride em **ciclos com pares de revisores**, seguindo rúbricas de qualidade.
6. O processo segue até **>= 80% dos Generais** concordarem que é a melhor implementação para o escopo solicitado.
7. A branch é promovida a **Pull Request**; **apenas o consenso dos Generais** pode aprovar o merge (facilitador executa o comando quando aprovado).
8. O ciclo continua para as próximas demandas, com histórico e métricas alimentando pesos dinâmicos e aprendizagem organizacional.

Esse fluxo é operacionalizado pela extensão (BIP-00) e respaldado pelas regras de consenso (vide Seções 4 e 5) e pela infraestrutura de votação (BIP-01).

### 12.1) Gate de Qualidade Pré-PR (obrigatório)
Antes de abrir a PR, após atingir **>= 80%** de aprovação dos Generais, o Hive deve garantir:

- **Documentação completa** da implementação (README modular, comentários de alto nível, ADRs quando aplicável).
- **Testes automatizados** cobrindo a classe/feature criada (unidade e, quando aplicável, integração), com metas mínimas de cobertura conforme Seção 5.
- **Lint e formatação** de acordo com o padrão do repositório; zero erros de linter.
- **Boas práticas**: design claro, responsabilidade única, erros tratados, logs adequados, limites de alocação/latência obedecidos se declarados.
- **Revalidação pelos Generais** focada em qualidade (não só no mérito da solução), mantendo **>= 80%** de aprovação.

Somente após esses critérios, a PR é aberta para **consenso final dos Generais** (facilitador executa merge quando aprovado).
