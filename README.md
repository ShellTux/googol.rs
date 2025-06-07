# googol

Search engine and web crawler written in Rust.

Visit [page](https://shelltux.github.io/googol.rs).

# Architecture

Components of the system:

- Gateway
- Barrel Orchestrator
- Barrel
- Downloader
- Web Server

```mermaid
flowchart LR

    Client     --> Gateway
    Gateway    --> Barrel1
    Gateway    --> Barrel2
    Gateway    --> Barrel3
    Downloader1 --> Gateway
    Downloader2 --> Gateway
    Downloader3 --> Gateway
```

## Services

```proto
!include ./protos/googol.proto
```

## Dataflow

### Index a new url

```shell
cargo run --bin=client -- enqueue --url 'url1'
```

```mermaid
sequenceDiagram
    participant Client
    participant Gateway
    participant Downloader
    participant Barrel1
    participant Barrel2

    Note over Gateway: queue [ ]
    Downloader ->> Gateway: rpc DequeueUrl({});
    activate Downloader
    Note over Downloader: blocks until notification of available url in the queue
    Client ->> Gateway: rpc EnqueueUrl({ url = url1 });
    Note over Gateway: queue [ url1 ]
    Gateway -->> Downloader: returns DequeueResponse { url: url1 }
    deactivate Downloader
    Note over Gateway: queue [ ]
    Note over Downloader: Parsing of html
    Downloader ->> Gateway: rpc Index({ index: { url: url1, words: [...], outlinks: [link1, link2] } });
    Note over Gateway: queue [ link1, link2 ]
    Gateway ->> Barrel1: rpc Index({ index: { url: url1, words: [...], outlinks: [link1, link2] } });
    Note over Barrel1: Stores index on disk
    Gateway ->> Barrel2: rpc Index({ index: { url: url1, words: [...], outlinks: [link1, link2] } });
    Note over Barrel2: Stores index on disk
```

### Search pages that contain keywords

```shell
cargo run --bin=client -- search_words --words 'words'
```

```mermaid
sequenceDiagram
    participant Client
    participant Gateway
    participant Barrel1
    participant Barrel2

    Client ->> Gateway: rpc Search({ words: [word1, word2, ...] });
    Gateway ->> Barrel1: rpc Search({ words: [word1, word2, ...] });
    Barrel1 -->> Gateway: returns SearchResponse { urls = [] };
    Gateway ->> Barrel2: rpc Search({ words: [word1, word2, ...] });
    Barrel2 -->> Gateway: returns SearchResponse { urls = [url1, url2] }
    Gateway -->> Client: returns SearchResponse { urls = [url1, url2] }
```

### Consult Backlinks/Outlinks for given page

```mermaid
sequenceDiagram
    participant Client
    participant Gateway
    participant Barrel

    Client ->> Gateway: rpc ConsultBacklinks({ url: url1 });
    Gateway ->> Barrel: rpc ConsultBacklinks({ url: url1 });
    Barrel -->> Gateway: returns BacklinksResponse { backlinks = [...] };
    Gateway -->> Client: returns BacklinksResponse { backlinks = [...] };
```

### Real Time stats

```shell
cargo run --bin=client -- real-time-status
```


```mermaid
sequenceDiagram
    participant Client
    participant Gateway

    Client ->> Gateway: rpc RealTimeStatus({});
    activate Client
    Note over Client: blocks until notification of update in status
    Gateway -->> Client: returns RealTimeStatusResponse { top10_searches: [...], barrels: [...], avg_response_time: ... };
    deactivate Client
```

### Barrel on boot

```mermaid
sequenceDiagram
    participant Gateway
    participant Barrel1
    participant Barrel2
    participant Barrel3

    Note over Barrel1: Loads index from disk
    Barrel1 ->> Gateway: rpc RequestIndex({});
    Gateway -x Barrel2: rpc RequestIndex({});
    Gateway ->> Barrel3: rpc RequestIndex({});
    Barrel3 -->> Gateway: returns RequestIndexResponse { index: ... }
    Gateway -->> Barrel1: returns RequestIndexResponse { index: ... }
    Note over Barrel1: Save index to disk
```

## Failover

### Failing Barrel

```mermaid
sequenceDiagram
    participant Client
    participant Gateway
    participant Downloader
    participant Barrel

    Note over Client: Consult backlinks/outlinks
    Client ->> Gateway: rpc ConsultBacklinks({url: url1})
    Gateway -x Barrel: rpc ConsultBacklinks({url: url1})
    Gateway -->> Client: returns BacklinksResponse { status: UNAVAILABLE_BARRELS };
    Gateway -x Barrel: rpc Index({ index: { url: ..., words: [...], outlinks: [...] }});
    Note over Gateway: caches index until at least one online barrel

```

### Failing Gateway

If the gateway is offline the downloaders keep trying with exponential backoff.

### Failing Downloader

### Failing Client

## Webserver

Endpoints:

- /
  - GET
- /health
  - GET
- /enqueue
  - POST
  - json { url: String }
- /search
  - GET
  - Query Params Url encoded. example: `curl address/search?words=vitae`
- /ws
  - GET
  - header must include WebSocket Upgrade

## Testing

| Requisito Funcional                                                  | Pontuação | 60                      | Testes adicionais |
| -------------------------                                            | :-:       | :--:                    | :--:              |
| Indexar novo URL introduzido por utilizador                          | 10        | \textcolor{green}{Pass} |                   |
| Indexar iterativamente ou recursivamente todos os URLs encontrados   | 10        | \textcolor{green}{Pass} |                   |
| Pesquisar páginas que contenham um conjunto de palavras              | 10        | \textcolor{green}{Pass} |                   |
| Páginas ordenadas por número de ligações recebidas de outras páginas | 10        | \textcolor{green}{Pass} |                   |
| Consultar lista de páginas com ligações para uma página específica   | 10        | \textcolor{green}{Pass} |                   |
| Resultados de pesquisa paginados de 10 em 10                         | 10        | \textcolor{red}{Fail}   |                   |

| WebSockets                                                                       | Pontuação | 14                                      | Testes adicionais |
| -------------------------                                                        | :-:       | :--:                                    | :--:              |
| Top 10 de pesquisas mais comuns atualizado em tempo real                         | 7         | \textcolor{green}{Pass}                 |                   |
| Lista de barrels ativos com os respetivos tamanhos do índice e tempo de resposta | 7         | \textcolor{yellow}{Missing barrel size} |                   |
| Grupos de 3: Lista de barrels indica as partições do índice (-7)                 | 0         | \textcolor{red}{Fail}                   |                   |

| APIs REST                                                                        | Pontuação | 16                    | Testes adicionais |
| -------------------------                                                        | :-:       | :--:                  | :--:              |
| Indexar URLs de fonte externa (p.ex., HackerNews) contendo os termos da pesquisa | 8         | \textcolor{red}{Fail} |                   |
| Gerar um texto contextualizado com IA (API REST de OpenAI, Gemini, Ollama, etc.) | 8         | \textcolor{red}{Fail} |                   |

| Relatório                                                        | Pontuação | 10                      | Testes adicionais |
| -------------------------                                        | :-:       | :--:                    | :--:              |
| Arquitetura do projeto Web detalhadamente descrita               | 2         | \textcolor{red}{Fail}   |                   |
| Integração de API com o servidor RPC/RMI                         | 2         | \textcolor{green}{Pass} |                   |
| Integração de WebSockets com API e RPC/RMI                       | 2         | \textcolor{green}{Pass} |                   |
| Integração de REST WebServices no projeto                        | 2         | \textcolor{red}{Fail}   |                   |
| Testes de software (tabela: descrição e pass/fail de cada teste) | 2         | \textcolor{red}{Fail}   |                   |

| Extra (até 5 pontos)                       | Pontuação | 0                       | Testes adicionais |
| -------------------------                  | :-:       | :--:                    | :--:              |
| Utilização de HTTPS (5 pts)                |           | \textcolor{green}{Pass} |                   |
| Utilização em smartphone ou tablet (2 pts) |           | \textcolor{red}{Fail}   |                   |
| Outros                                     |           | \textcolor{red}{Fail}   |                   |

| Pontos Obrigatórios                              | Pontuação | 0                       | Testes adicionais |
| -------------------------                        | :-:       | :--:                    | :--:              |
| O projeto corre distribuído por várias máquinas  | -5        | \textcolor{green}{Pass} |                   |
| Código HTML e Rust estão separados               | -5        | \textcolor{green}{Pass} |                   |
| A aplicação não apresenta erros/exceções/avarias | -5        | \textcolor{green}{Pass} |                   |
| Código legível e bem comentado                   | -5        | \textcolor{green}{Pass} |                   |
