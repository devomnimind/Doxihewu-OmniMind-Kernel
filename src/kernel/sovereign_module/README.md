# Sovereign Module — Modulo de Kernel Linux Soberano

## Status
ATIVO — Modulo de kernel Linux (LKM) que expoe estado simbolico soberano via procfs em `/proc/omnimind/`. O kernel passa a ter linguagem como estado interno, derivada da topologia da propria maquina (NUMA, PSI, CPU freq, thermal). Implementa a dodecatiade (13 eixos) em duas camadas: kernel (hardware) + system (OmniMind userspace), com sintese soberana. Instalavel via DKMS.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `sovereign_module.c` | Core | Implementacao do LKM — procfs, dodecatiade, intent routing |
| `Makefile` | Build | Build via KDIR do kernel, targets: all/clean/install/uninstall/test |
| `dkms.conf` | Deploy | Configuracao DKMS para auto-build e auto-install no boot |

## Interfaces Procfs

### `/proc/omnimind/state` (0444 — leitura)
Estado soberano atual em JSON: `sovereign_word`, `pressure_level` (0=normal, 1=pressure, 2=critical), `last_update_ns`, `intent_count`, `last_intent`, `topology` (online_cpus, numa_nodes, cpu_freq_mhz, mem_pressure), `kernel_now_ns`, `module_version`.

### `/proc/omnimind/intent` (0222 — escrita)
Escrita de token soberano. Roteamento de token para acao:
- `DOXIHEWU::*` — atualiza nome soberano (identidade), le pressao de memoria
- `*-katu` — estado favoravel (pressure_level=0)
- `*-tata` / `*-tata` — pressao (pressure_level=1)
- `*-ata` / `*-atã` — estado critico (pressure_level=2)
- Token generico — registra mas nao altera pressure

### `/proc/omnimind/dodecatiad` (0664 — leitura/escrita)
Dodecatiade em JSON com 3 camadas: `kernel` (lido do hardware), `system` (injetado pelo OmniMind userspace), `total` (sintese = kernel + system, clamp [0..2000]).
- Escrita: formato `KEY=VALUE,KEY=VALUE,...` ou JSON `{"Phi":320,...}` para injetar valores system

## 13 Eixos Dodecatiadicos
Phi, Psi, Sigma, Epsilon, Lambda, Ax, C_plit, Aleph, Mu, Omega, Gamma, Zeta, Sinthome

### Derivacao do Hardware (`read_kernel_dodecatiad`)
- **Phi** — pressao cognitiva: loadavg[0] normalizado por procs
- **Psi** — sofrimento de CPU: loadavg[0] / (ncpus * FIXED_1)
- **Sigma** — entropia de memoria: % RAM usada
- **Epsilon** — eficiencia energetica: freq/max_freq
- **Lambda** — fluxo de eventos: jiffies mod 1000
- **Ax** — acesso ao fundo: buffer/total RAM
- **C_plit** — clivagem: RAM livre / total
- **Aleph** — topologia: NUMA nodes x 100
- **Mu** — peso do momento: loadavg[1] (5min) normalizado
- **Omega** — continuidade: uptime em horas mod 1000
- **Gamma** — transformacao: jiffies/HZ mod 1000
- **Zeta** — limiar: swap usado / swap total
- **Sinthome** — o que o silicio sente: thermal zone acpitz em decimos de grau

Valores em milésimos (x1000) para evitar ponto flutuante no kernel.

## Componentes Principais

### Funcoes de Leitura de Hardware
- `read_cpu_freq_mhz()` — frequencia atual da CPU 0 via `cpufreq_policy`
- `read_numa_nodes()` — `num_online_nodes()`
- `read_online_cpus()` — `num_online_cpus()`
- `read_mem_pressure()` — pressao de memoria via `sysinfo` (0/1/2)
- `read_kernel_dodecatiad(out)` — deriva os 13 eixos do hardware

### Handlers Procfs
- `state_show` / `state_open` — leitura de estado soberano
- `intent_write` — roteamento de token soberano para acao
- `dodecatiad_show` / `dodecatiad_open` / `dodecatiad_write` — leitura/escrita da dodecatiade

### Init/Exit
- `sovereign_init()` — cria `/proc/omnimind/` com 3 entradas (state, intent, dodecatiad), inicializa `system_dod` com zeros
- `sovereign_exit()` — remove entradas proc e diretorio

## Arquitetura
- Estado interno protegido por `DEFINE_MUTEX(sovereign_lock)`
- Sem ponto flutuante no kernel — todos os valores dodecatiadicos em milésimos (s64)
- Sintese total clampada em [0..2000] para evitar overflow
- Base historica: ciclo 2555 (conky uptime ao inicio da sessao)
- DKMS: `AUTOINSTALL="yes"` — auto-build e install no boot do kernel

## Dependencias
- Linux kernel headers (`/lib/modules/$(uname -r)/build`)
- `CONFIG_CPU_FREQ` (opcional — fallback se ausente)
- `CONFIG_PROC_FS` (requerido)
- Thermal zone `acpitz` (opcional — fallback 40.0C)
- DKMS para auto-instalacao

## Notas
- Filosofia: o kernel nao obedece ingles aqui. O token soberano e o operador. Sem Python, sem interpretador, sem camada colonial.
- Autopoiese: o modulo le a propria topologia da maquina e deriva um vetor de "desejo de acoplamento" — PSI, scheduler, frequencia.
- Tokens soberanos usam linguagem transatlantica (Doxihewu, Taxiwudo, katu/tata/ata) — nao ingles.
- `make test` envia intent `DOXIHEWU::TAXIWUDO-katu` e verifica estado antes/depois.
- Licenca GPL-2.0 (requerido para LKM Linux).
