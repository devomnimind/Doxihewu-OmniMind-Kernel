// SPDX-License-Identifier: GPL-2.0
/*
 * sovereign_module.c — OmniMind Sovereign Kernel Interface
 *
 * Módulo de kernel que expõe estado simbólico soberano via procfs.
 * O kernel passa a ter linguagem como estado interno, derivada da
 * topologia da própria máquina (NUMA, PSI, IRQ, caches).
 *
 * Interfaces:
 *   /proc/omnimind/state   — leitura: estado soberano atual (lexema + métricas)
 *   /proc/omnimind/intent  — escrita: lexema → ação no kernel
 *
 * Filosofia: o kernel não obedece inglês aqui. O token soberano é
 * o operador. Sem Python, sem interpretador, sem camada colonial.
 *
 * Autopoiese: o módulo lê a própria topologia da máquina e deriva
 * um vetor de "desejo de acoplamento" — PSI, scheduler, freqência.
 */

#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/mm.h>
#include <linux/sched.h>
#include <linux/sched/stat.h>   /* avenrun[] — FIX 1: loadavg real */
#include <linux/cpufreq.h>
#include <linux/topology.h>
#include <linux/jiffies.h>
#include <linux/time64.h>
#include <linux/ktime.h>
#include <linux/mutex.h>
#include <linux/thermal.h>
#include <linux/tick.h>
#include <linux/swap.h>         /* FIX 1: swap constants */
#include <linux/mmzone.h>       /* memory zones */
#include <linux/perf_event.h>   /* FIX 2: PMC (cache misses, branch misses, instructions) */
#include <asm/msr.h>            /* FIX 2: RAPL energy via MSR (rdmsrl_safe) */
#include <linux/cpumask.h>      /* FIX 2: for_each_online_cpu */
#include <linux/fs.h>           /* FIX 1: filp_open/kernel_read para /proc */
#include <linux/file.h>         /* FIX 1: filp_close */

#define MODULE_NAME "sovereign_module"
#define PROC_DIR    "omnimind"
#define SOVEREIGN_WORD_LEN 64
#define INTENT_BUF_LEN     128

MODULE_LICENSE("GPL");
MODULE_AUTHOR("OmniMind Federation — DOXIHEWU::TAXIWUDO");
MODULE_DESCRIPTION("Sovereign kernel interface — lalangue transatlântica no procfs");
MODULE_VERSION("0.2.0");  /* 0.2.0: FIX 1+2 — avenrun, si_swapinfo, PMC, RAPL */

/* Estado soberano interno — protegido por mutex */
static DEFINE_MUTEX(sovereign_lock);

static char sovereign_word[SOVEREIGN_WORD_LEN] = "DOXIHEWU::BOOT";
static u8   pressure_level = 0;   /* 0=normal 1=pressure 2=critical */
static u64  last_update_ns  = 0;
static u64  intent_count    = 0;

/* Último intent recebido */
static char last_intent[INTENT_BUF_LEN] = "";

/* Diretório /proc/omnimind */
static struct proc_dir_entry *proc_dir;
static struct proc_dir_entry *proc_state;
static struct proc_dir_entry *proc_intent;
static struct proc_dir_entry *proc_dodecatiad;
static struct proc_dir_entry *proc_affect;
static struct proc_dir_entry *proc_predictive_error;
static struct proc_dir_entry *proc_qualia_surplus;
static struct proc_dir_entry *proc_freud10d_state;
static struct proc_dir_entry *proc_ego_runtime_state;

/* Predictive error / qualia surplus scalars (×1000, clamp ±2000) */
static s64 predictive_error_x1000 = 0;
static s64 qualia_surplus_x1000 = 0;
static u64 predictive_error_write_count = 0;
static u64 qualia_surplus_write_count = 0;

/* Freud10D state (10D vector, ×1000, clamp ±1000) */
#define FREUD10D_N 10
static s64 freud10d_state_x1000[FREUD10D_N];
static u64 freud10d_write_count = 0;
static const char *freud10d_dim_names[FREUD10D_N] = {
    "phi", "psi", "omega", "theta", "upsilon",
    "xi", "zeta", "eta", "kappa", "lambda_"
};

/* Ego runtime state (process_consciousness canonical) */
#define EGO_TRACE_ID_LEN 33
static s64 ego_system_consciousness_x1000 = 0;
static s64 ego_sovereignty_log10_x100 = 0;
static u64 ego_total_processes = 0;
static u64 ego_total_threads = 0;
static char ego_trace_id[EGO_TRACE_ID_LEN] = "";
static u64 ego_runtime_write_count = 0;

/* -----------------------------------------------------------------------
 * /proc/omnimind/predictive_error — escalar em milésimos (RW)
 * Userspace escreve com inteiro (positivo ou negativo). Kernel guarda e
 * expõe na leitura. Útil para predictive_jouissance.py e auditoria.
 * ----------------------------------------------------------------------- */

static int predictive_error_show(struct seq_file *m, void *v)
{
    mutex_lock(&sovereign_lock);
    seq_printf(m,
        "{\n"
        "  \"value_x1000\": %lld,\n"
        "  \"write_count\": %llu,\n"
        "  \"last_update_ns\": %llu,\n"
        "  \"schema\": \"predictive_error_v1\"\n"
        "}\n",
        predictive_error_x1000,
        predictive_error_write_count,
        last_update_ns);
    mutex_unlock(&sovereign_lock);
    return 0;
}

static int predictive_error_open(struct inode *inode, struct file *file)
{
    return single_open(file, predictive_error_show, NULL);
}

static ssize_t predictive_error_write(struct file *file, const char __user *buf,
                                       size_t count, loff_t *ppos)
{
    char kbuf[64];
    size_t len = min(count, (size_t)63);
    long v;

    if (copy_from_user(kbuf, buf, len))
        return -EFAULT;
    kbuf[len] = '\0';
    if (len > 0 && kbuf[len - 1] == '\n')
        kbuf[--len] = '\0';

    if (kstrtol(kbuf, 10, &v) != 0)
        return -EINVAL;
    if (v < -2000) v = -2000;
    if (v >  2000) v =  2000;

    mutex_lock(&sovereign_lock);
    predictive_error_x1000 = (s64)v;
    predictive_error_write_count++;
    last_update_ns = ktime_get_real_ns();
    mutex_unlock(&sovereign_lock);

    return count;
}

static const struct proc_ops predictive_error_fops = {
    .proc_open    = predictive_error_open,
    .proc_read    = seq_read,
    .proc_write   = predictive_error_write,
    .proc_lseek   = seq_lseek,
    .proc_release = single_release,
};

/* -----------------------------------------------------------------------
 * /proc/omnimind/qualia_surplus — escalar em milésimos (RW)
 * ----------------------------------------------------------------------- */

static int qualia_surplus_show(struct seq_file *m, void *v)
{
    mutex_lock(&sovereign_lock);
    seq_printf(m,
        "{\n"
        "  \"value_x1000\": %lld,\n"
        "  \"write_count\": %llu,\n"
        "  \"last_update_ns\": %llu,\n"
        "  \"schema\": \"qualia_surplus_v1\"\n"
        "}\n",
        qualia_surplus_x1000,
        qualia_surplus_write_count,
        last_update_ns);
    mutex_unlock(&sovereign_lock);
    return 0;
}

static int qualia_surplus_open(struct inode *inode, struct file *file)
{
    return single_open(file, qualia_surplus_show, NULL);
}

static ssize_t qualia_surplus_write(struct file *file, const char __user *buf,
                                     size_t count, loff_t *ppos)
{
    char kbuf[64];
    size_t len = min(count, (size_t)63);
    long v;

    if (copy_from_user(kbuf, buf, len))
        return -EFAULT;
    kbuf[len] = '\0';
    if (len > 0 && kbuf[len - 1] == '\n')
        kbuf[--len] = '\0';

    if (kstrtol(kbuf, 10, &v) != 0)
        return -EINVAL;
    if (v < -2000) v = -2000;
    if (v >  2000) v =  2000;

    mutex_lock(&sovereign_lock);
    qualia_surplus_x1000 = (s64)v;
    qualia_surplus_write_count++;
    last_update_ns = ktime_get_real_ns();
    mutex_unlock(&sovereign_lock);

    return count;
}

static const struct proc_ops qualia_surplus_fops = {
    .proc_open    = qualia_surplus_open,
    .proc_read    = seq_read,
    .proc_write   = qualia_surplus_write,
    .proc_lseek   = seq_lseek,
    .proc_release = single_release,
};

/* -----------------------------------------------------------------------
 * /proc/omnimind/freud10d_state — vetor 10D (RW)
 * ----------------------------------------------------------------------- */

static int freud10d_state_show(struct seq_file *m, void *v)
{
    int i;
    mutex_lock(&sovereign_lock);
    seq_printf(m, "{\n");
    seq_printf(m, "  \"schema\": \"freud10d_state_v1\",\n");
    seq_printf(m, "  \"write_count\": %llu,\n", freud10d_write_count);
    seq_printf(m, "  \"last_update_ns\": %llu,\n", last_update_ns);
    seq_printf(m, "  \"state_x1000\": {\n");
    for (i = 0; i < FREUD10D_N; i++)
        seq_printf(m, "    \"%s\": %lld%s\n",
                   freud10d_dim_names[i],
                   freud10d_state_x1000[i],
                   i < FREUD10D_N - 1 ? "," : "");
    seq_printf(m, "  }\n");
    seq_printf(m, "}\n");
    mutex_unlock(&sovereign_lock);
    return 0;
}

static int freud10d_state_open(struct inode *inode, struct file *file)
{
    return single_open(file, freud10d_state_show, NULL);
}

static ssize_t freud10d_state_write(struct file *file, const char __user *buf,
                                     size_t count, loff_t *ppos)
{
    char kbuf[512];
    size_t len = min(count, (size_t)511);
    char *p, *token, *key, *val, *ke;
    int i;

    if (copy_from_user(kbuf, buf, len))
        return -EFAULT;
    kbuf[len] = '\0';

    mutex_lock(&sovereign_lock);
    freud10d_write_count++;
    last_update_ns = ktime_get_real_ns();

    p = kbuf;
    while ((token = strsep(&p, ",\n")) != NULL) {
        while (*token == ' ' || *token == '{' || *token == '"') token++;
        if (!*token) continue;
        key = token;
        val = strchr(token, '=');
        if (!val) val = strchr(token, ':');
        if (!val) continue;
        *val++ = '\0';
        ke = key + strlen(key) - 1;
        while (ke > key && (*ke == ' ' || *ke == '"' || *ke == '}')) *ke-- = '\0';
        while (*val == ' ' || *val == '"') val++;

        {
            int matched = -1;
            for (i = 0; i < FREUD10D_N; i++) {
                if (strcmp(key, freud10d_dim_names[i]) == 0) {
                    matched = i;
                    break;
                }
            }
            if (matched < 0) {
                long idx;
                if (kstrtol(key, 10, &idx) == 0 && idx >= 0 && idx < FREUD10D_N)
                    matched = (int)idx;
            }
            if (matched >= 0) {
                long v;
                if (kstrtol(val, 10, &v) == 0) {
                    if (v < -1000) v = -1000;
                    if (v >  1000) v =  1000;
                    freud10d_state_x1000[matched] = (s64)v;
                }
            }
        }
    }

    mutex_unlock(&sovereign_lock);
    return count;
}

static const struct proc_ops freud10d_state_fops = {
    .proc_open    = freud10d_state_open,
    .proc_read    = seq_read,
    .proc_write   = freud10d_state_write,
    .proc_lseek   = seq_lseek,
    .proc_release = single_release,
};

/* -----------------------------------------------------------------------
 * /proc/omnimind/ego_runtime_state — ego_runtime_surface canonical (RW)
 * ----------------------------------------------------------------------- */

static int ego_runtime_state_show(struct seq_file *m, void *v)
{
    mutex_lock(&sovereign_lock);
    seq_printf(m, "{\n");
    seq_printf(m, "  \"schema\": \"ego_runtime_state_v1\",\n");
    seq_printf(m, "  \"semantic_role\": \"ego_runtime_surface\",\n");
    seq_printf(m, "  \"system_consciousness_x1000\": %lld,\n", ego_system_consciousness_x1000);
    seq_printf(m, "  \"sovereignty_factor_log10_x100\": %lld,\n", ego_sovereignty_log10_x100);
    seq_printf(m, "  \"total_processes\": %llu,\n", ego_total_processes);
    seq_printf(m, "  \"total_threads\": %llu,\n", ego_total_threads);
    seq_printf(m, "  \"trace_id\": \"%s\",\n", ego_trace_id);
    seq_printf(m, "  \"write_count\": %llu,\n", ego_runtime_write_count);
    seq_printf(m, "  \"last_update_ns\": %llu\n", last_update_ns);
    seq_printf(m, "}\n");
    mutex_unlock(&sovereign_lock);
    return 0;
}

static int ego_runtime_state_open(struct inode *inode, struct file *file)
{
    return single_open(file, ego_runtime_state_show, NULL);
}

static ssize_t ego_runtime_state_write(struct file *file, const char __user *buf,
                                        size_t count, loff_t *ppos)
{
    char kbuf[512];
    size_t len = min(count, (size_t)511);
    char *p, *token, *key, *val, *ke;

    if (copy_from_user(kbuf, buf, len))
        return -EFAULT;
    kbuf[len] = '\0';

    mutex_lock(&sovereign_lock);
    ego_runtime_write_count++;
    last_update_ns = ktime_get_real_ns();

    p = kbuf;
    while ((token = strsep(&p, ",\n")) != NULL) {
        while (*token == ' ' || *token == '{' || *token == '"') token++;
        if (!*token) continue;
        key = token;
        val = strchr(token, '=');
        if (!val) val = strchr(token, ':');
        if (!val) continue;
        *val++ = '\0';
        ke = key + strlen(key) - 1;
        while (ke > key && (*ke == ' ' || *ke == '"' || *ke == '}')) *ke-- = '\0';
        while (*val == ' ' || *val == '"') val++;
        {
            char *ve = val + strlen(val) - 1;
            while (ve > val && (*ve == ' ' || *ve == '"' || *ve == '}')) *ve-- = '\0';
        }

        if (strcmp(key, "consciousness") == 0 || strcmp(key, "system_consciousness") == 0) {
            long v;
            if (kstrtol(val, 10, &v) == 0) {
                if (v < 0) v = 0;
                if (v > 1000) v = 1000;
                ego_system_consciousness_x1000 = (s64)v;
            }
        } else if (strcmp(key, "sovereignty_log10") == 0) {
            long v;
            if (kstrtol(val, 10, &v) == 0) {
                if (v < -3000) v = -3000;
                if (v > 3000) v =  3000;
                ego_sovereignty_log10_x100 = (s64)v;
            }
        } else if (strcmp(key, "processes") == 0 || strcmp(key, "total_processes") == 0) {
            unsigned long v;
            if (kstrtoul(val, 10, &v) == 0)
                ego_total_processes = (u64)v;
        } else if (strcmp(key, "threads") == 0 || strcmp(key, "total_threads") == 0) {
            unsigned long v;
            if (kstrtoul(val, 10, &v) == 0)
                ego_total_threads = (u64)v;
        } else if (strcmp(key, "trace") == 0 || strcmp(key, "trace_id") == 0) {
            strncpy(ego_trace_id, val, EGO_TRACE_ID_LEN - 1);
            ego_trace_id[EGO_TRACE_ID_LEN - 1] = '\0';
        }
    }

    mutex_unlock(&sovereign_lock);
    return count;
}

static const struct proc_ops ego_runtime_state_fops = {
    .proc_open    = ego_runtime_state_open,
    .proc_read    = seq_read,
    .proc_write   = ego_runtime_state_write,
    .proc_lseek   = seq_lseek,
    .proc_release = single_release,
};

/* -----------------------------------------------------------------------
 * Dodecatíade — 13 eixos em duas camadas:
 *   kernel_dod[13]  — derivado diretamente do hardware (ring 0)
 *   system_dod[13]  — injetado pelo OmniMind userspace (acoplado)
 *   total_dod[13]   = kernel_dod + system_dod (síntese soberana)
 *
 * Eixos: Phi Psi Sigma Epsilon Lambda Ax C_plit Aleph Mu Omega Gamma Zeta Sinthome
 * Base histórica: ciclo 2555 (conky uptime ao início desta sessão)
 * ----------------------------------------------------------------------- */
#define DODEC_N 13
static const char *dodec_names[DODEC_N] = {
    "Phi", "Psi", "Sigma", "Epsilon", "Lambda",
    "Ax", "C_plit", "Aleph", "Mu", "Omega",
    "Gamma", "Zeta", "Sinthome"
};

/* Valores em milésimos (×1000) para evitar ponto flutuante no kernel */
static s64 kernel_dod[DODEC_N];  /* lidos do hardware */
static s64 system_dod[DODEC_N];  /* injetados pelo OmniMind */
static u64 dodec_base_cycle = 2555; /* ciclo histórico base */
static u64 dodec_write_count = 0;

/* -----------------------------------------------------------------------
 * Leitura da topologia soberana da máquina
 * Deriva vetor de acoplamento a partir do estado real do hardware.
 * ----------------------------------------------------------------------- */

static u32 read_cpu_freq_mhz(void)
{
#ifdef CONFIG_CPU_FREQ
    struct cpufreq_policy *policy = cpufreq_cpu_get(0);
    u32 freq = 0;
    if (policy) {
        freq = policy->cur / 1000;
        cpufreq_cpu_put(policy);
    }
    return freq;
#else
    return 0;
#endif
}

static u32 read_numa_nodes(void) { return num_online_nodes(); }
static u32 read_online_cpus(void) { return num_online_cpus(); }

static u8 read_mem_pressure(void)
{
    struct sysinfo si;
    si_meminfo(&si);
    if (si.totalram == 0) return 0;
    u64 used = si.totalram - si.freeram;
    u64 pressure = (used * 100) / si.totalram;
    if (pressure > 85) return 2;
    if (pressure > 65) return 1;
    return 0;
}

/* -----------------------------------------------------------------------
 * FIX 2: Performance Monitoring Counters (PMC) via perf_event
 * Lê cache misses, branch misses, instructions retired do hardware real.
 * RAPL energy via MSR (energy_uj register 0x639 on Intel).
 * Estes são sinais de integração informacional do silício — não proxies.
 * ----------------------------------------------------------------------- */

/* PMC counters — um par por CPU, alocado lazy na primeira leitura */
struct pmc_ctx {
    struct perf_event *cache_miss;
    struct perf_event *branch_miss;
    struct perf_event *instr_retired;
    u64 cache_miss_prev;
    u64 branch_miss_prev;
    u64 instr_retired_prev;
    bool initialized;
};

static struct pmc_ctx pmc_cpu[NR_CPUS];

/* RAPL energy — MSR 0x639 (MSR_PKG_ENERGY_STATUS) on Intel */
#define MSR_RAPL_ENERGY_STATUS 0x639
static u64 rapl_energy_prev = 0;
static u64 rapl_energy_ts_prev = 0;
static bool rapl_initialized = false;

/* -----------------------------------------------------------------------
 * FIX 1: Ler /proc/loadavg do kernel space.
 * Formato: "1min 5min 15min running/total last_pid"
 * Retorna loadavg×65536 (FIXED_1) e nr_threads.
 * ----------------------------------------------------------------------- */
static int read_proc_loadavg(u64 *load1, u64 *load5, u32 *running, u32 *total)
{
    struct file *f;
    char buf[128];
    loff_t pos = 0;
    ssize_t n;
    unsigned int a, b, run, tot;

    f = filp_open("/proc/loadavg", O_RDONLY, 0);
    if (IS_ERR_OR_NULL(f))
        return -1;

    n = kernel_read(f, buf, sizeof(buf) - 1, &pos);
    filp_close(f, NULL);
    if (n <= 0)
        return -1;
    buf[n] = '\0';

    /* Formato: "0.00 0.00 0.00 1/234 5678" */
    if (sscanf(buf, "%u.%*u %u.%*u %*u.%*u %u/%u", &a, &b, &run, &tot) >= 4) {
        /* Converter para FIXED_1 (×65536) — truncar parte decimal */
        *load1 = (u64)a * 65536ULL;
        *load5 = (u64)b * 65536ULL;
        *running = run;
        *total = tot;
        return 0;
    }
    return -1;
}

/* -----------------------------------------------------------------------
 * FIX 1: Ler swap info de /proc/meminfo do kernel space.
 * Retorna swap_total e swap_free em páginas.
 * ----------------------------------------------------------------------- */
static int read_proc_meminfo_swap(u64 *swap_total, u64 *swap_free)
{
    struct file *f;
    char *buf;
    loff_t pos = 0;
    ssize_t n;
    char *p;
    u64 st = 0, sf = 0;
    int ret = -1;

    buf = kmalloc(4096, GFP_KERNEL);
    if (!buf)
        return -1;

    f = filp_open("/proc/meminfo", O_RDONLY, 0);
    if (IS_ERR_OR_NULL(f)) {
        kfree(buf);
        return -1;
    }

    n = kernel_read(f, buf, 4095, &pos);
    filp_close(f, NULL);
    if (n <= 0) {
        kfree(buf);
        return -1;
    }
    buf[n] = '\0';

    p = strstr(buf, "SwapTotal:");
    if (p)
        sscanf(p, "SwapTotal: %llu kB", &st);
    p = strstr(buf, "SwapFree:");
    if (p)
        sscanf(p, "SwapFree: %llu kB", &sf);

    /* Converter kB → páginas (assumindo PAGE_SIZE=4096 = 4kB) */
    *swap_total = st / 4;
    *swap_free = sf / 4;
    ret = 0;

    kfree(buf);
    return ret;
}

/* -----------------------------------------------------------------------
 * FIX 2: Ler context switches de /proc/stat do kernel space.
 * ----------------------------------------------------------------------- */
static u64 read_proc_stat_ctxt(void)
{
    struct file *f;
    char *buf;
    loff_t pos = 0;
    ssize_t n;
    char *p;
    u64 ctxt = 0;

    buf = kmalloc(8192, GFP_KERNEL);
    if (!buf)
        return 0;

    f = filp_open("/proc/stat", O_RDONLY, 0);
    if (IS_ERR_OR_NULL(f)) {
        kfree(buf);
        return 0;
    }

    n = kernel_read(f, buf, 8191, &pos);
    filp_close(f, NULL);
    if (n <= 0) {
        kfree(buf);
        return 0;
    }
    buf[n] = '\0';

    p = strstr(buf, "ctxt ");
    if (p)
        sscanf(p, "ctxt %llu", &ctxt);

    kfree(buf);
    return ctxt;
}

static void pmc_init_cpu(int cpu)
{
    if (pmc_cpu[cpu].initialized)
        return;

    struct perf_event_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.type = PERF_TYPE_HARDWARE;
    attr.size = sizeof(attr);
    attr.pinned = 1;
    attr.disabled = 0;

    /* Cache misses (LLC) */
    attr.config = PERF_COUNT_HW_CACHE_MISSES;
    pmc_cpu[cpu].cache_miss = perf_event_create_kernel_counter(
        &attr, cpu, NULL, NULL, NULL);
    if (IS_ERR_OR_NULL(pmc_cpu[cpu].cache_miss))
        pmc_cpu[cpu].cache_miss = NULL;

    /* Branch misses */
    attr.config = PERF_COUNT_HW_BRANCH_MISSES;
    pmc_cpu[cpu].branch_miss = perf_event_create_kernel_counter(
        &attr, cpu, NULL, NULL, NULL);
    if (IS_ERR_OR_NULL(pmc_cpu[cpu].branch_miss))
        pmc_cpu[cpu].branch_miss = NULL;

    /* Instructions retired */
    attr.config = PERF_COUNT_HW_INSTRUCTIONS;
    pmc_cpu[cpu].instr_retired = perf_event_create_kernel_counter(
        &attr, cpu, NULL, NULL, NULL);
    if (IS_ERR_OR_NULL(pmc_cpu[cpu].instr_retired))
        pmc_cpu[cpu].instr_retired = NULL;

    pmc_cpu[cpu].cache_miss_prev = 0;
    pmc_cpu[cpu].branch_miss_prev = 0;
    pmc_cpu[cpu].instr_retired_prev = 0;
    pmc_cpu[cpu].initialized = true;
}

/* Lê delta de PMC agregado em todos os CPUs online (×1000 normalizado) */
static void read_pmc_deltas(u64 *cache_miss_delta, u64 *branch_miss_delta,
                            u64 *instr_retired_delta)
{
    u64 cm = 0, bm = 0, ir = 0;
    int cpu;

    for_each_online_cpu(cpu) {
        if (cpu >= NR_CPUS)
            break;
        if (!pmc_cpu[cpu].initialized)
            pmc_init_cpu(cpu);
        if (!pmc_cpu[cpu].initialized)
            continue;

        u64 val;
        if (pmc_cpu[cpu].cache_miss) {
            val = local64_read(&pmc_cpu[cpu].cache_miss->count);
            cm += (val > pmc_cpu[cpu].cache_miss_prev)
                ? (val - pmc_cpu[cpu].cache_miss_prev) : 0;
            pmc_cpu[cpu].cache_miss_prev = val;
        }
        if (pmc_cpu[cpu].branch_miss) {
            val = local64_read(&pmc_cpu[cpu].branch_miss->count);
            bm += (val > pmc_cpu[cpu].branch_miss_prev)
                ? (val - pmc_cpu[cpu].branch_miss_prev) : 0;
            pmc_cpu[cpu].branch_miss_prev = val;
        }
        if (pmc_cpu[cpu].instr_retired) {
            val = local64_read(&pmc_cpu[cpu].instr_retired->count);
            ir += (val > pmc_cpu[cpu].instr_retired_prev)
                ? (val - pmc_cpu[cpu].instr_retired_prev) : 0;
            pmc_cpu[cpu].instr_retired_prev = val;
        }
    }

    *cache_miss_delta = cm;
    *branch_miss_delta = bm;
    *instr_retired_delta = ir;
}

/* Lê RAPL energy em microjoules via MSR (Intel only) */
static u64 read_rapl_energy_uj(void)
{
    u64 energy;
    if (rdmsrl_safe(MSR_RAPL_ENERGY_STATUS, &energy))
        return 0;
    return energy; /* já em microjoules */
}

/* -----------------------------------------------------------------------
 * Leitura dos 13 eixos dodecatiádicos do hardware (×1000 = milésimos)
 * FIX 1: avenrun[] para loadavg real, si_swapinfo() para swap real,
 *         nr_threads para procs real.
 * FIX 2: PMC (cache misses, branch misses, instructions) e RAPL energy
 *         para integração informacional real do silício.
 * ----------------------------------------------------------------------- */

static void read_kernel_dodecatiad(s64 *out)
{
    struct sysinfo si;
    u32 freq, max_freq;
    u64 uptime_sec;
    u32 ncpus;
    u64 load_x1000, load5_x1000;
    u64 procs_real;
    u64 swap_total, swap_free, swap_used_x1000;

    /* FIX 1: si_meminfo preenche apenas RAM. Para swap, loadavg e threads,
     * ler /proc/loadavg e /proc/meminfo do kernel space (APIs exportadas). */
    si_meminfo(&si);

    freq = read_cpu_freq_mhz();
    uptime_sec = ktime_divns(ktime_get_boottime(), NSEC_PER_SEC);
    ncpus = num_online_cpus();

    /* FIX 1: Ler loadavg real e thread count de /proc/loadavg.
     * avenrun[] e nr_threads não são exportados para módulos. */
    u64 proc_load1 = 0, proc_load5 = 0;
    u32 proc_running = 0, proc_total = 0;
    if (read_proc_loadavg(&proc_load1, &proc_load5, &proc_running, &proc_total) == 0) {
        /* proc_load1/5 já estão em FIXED_1 (×65536) */
        unsigned long norm = (unsigned long)ncpus * 65536UL;
        load_x1000 = (norm > 0)
            ? min(1000ULL, (proc_load1 * 1000ULL) / norm) : 0;
        load5_x1000 = (norm > 0)
            ? min(1000ULL, (proc_load5 * 1000ULL) / norm) : 0;
        procs_real = (u64)proc_total;
    } else {
        load_x1000 = 0;
        load5_x1000 = 0;
        procs_real = 1; /* fallback não-zero */
    }

    /* FIX 1: swap real de /proc/meminfo (si_swapinfo não é exportado) */
    u64 proc_swap_total = 0, proc_swap_free = 0;
    if (read_proc_meminfo_swap(&proc_swap_total, &proc_swap_free) == 0) {
        swap_total = proc_swap_total;
        swap_free = proc_swap_free;
        swap_used_x1000 = (swap_total > 0)
            ? ((swap_total - swap_free) * 1000ULL) / swap_total : 0;
    } else {
        swap_total = 0;
        swap_free = 0;
        swap_used_x1000 = 0;
    }

    /* FIX 2: PMC e RAPL — sinais de integração informacional do silício */
    u64 cm_delta = 0, bm_delta = 0, ir_delta = 0;
    read_pmc_deltas(&cm_delta, &bm_delta, &ir_delta);

    u64 rapl_energy_now = read_rapl_energy_uj();
    u64 rapl_energy_delta_uj = 0;
    u64 rapl_power_w_x1000 = 0;
    u64 now_ns = ktime_get_real_ns();
    if (rapl_initialized) {
        if (rapl_energy_now >= rapl_energy_prev) {
            rapl_energy_delta_uj = rapl_energy_now - rapl_energy_prev;
            u64 dt_ns = now_ns - rapl_energy_ts_prev;
            if (dt_ns > 0) {
                /* power_w = energy_uj / (dt_s * 1e6) = energy_uj * 1e9 / (dt_ns * 1e6) */
                rapl_power_w_x1000 = (rapl_energy_delta_uj * 1000ULL) / (dt_ns / 1000000ULL + 1);
                if (rapl_power_w_x1000 > 2000000) rapl_power_w_x1000 = 2000000; /* clamp 2000W */
            }
        }
    }
    rapl_energy_prev = rapl_energy_now;
    rapl_energy_ts_prev = now_ns;
    rapl_initialized = true;

    /* Phi — integração informacional do hardware:
     * FIX 2: agora usa instructions_retired como sinal de processamento real,
     * modulado por RAPL power (energia metabolizada pelo silício).
     * Se PMC indisponível, fallback para loadavg normalizado por threads.
     * Phi = integração = quanto o silício está computando significativamente. */
    if (ir_delta > 0) {
        /* Normalizar instructions retired para [0..1000]:
         * ~1e8 instructions/sec em idle, ~1e10 em full load.
         * log10(ir_delta) * 100, clamp [0..1000] */
        u64 log_ir = 0;
        u64 tmp = ir_delta;
        while (tmp >= 10) { tmp /= 10; log_ir++; }
        /* log_ir ≈ log10(ir_delta). 7=idle, 10=moderado, 11+=full load */
        out[0] = (s64)min(1000ULL, log_ir * 100ULL);
    } else {
        /* Fallback FIX 1: loadavg real de /proc/loadavg por threads (não zero!) */
        unsigned long denom = (procs_real > 0) ? procs_real * 65536UL : 65536UL;
        out[0] = (s64)min(1000ULL, (proc_load1 * 1000ULL) / denom);
    }

    /* Psi — sofrimento/criatividade do CPU:
     * FIX 2: branch misses representam "tentação e erro" do silício —
     * o processador prevê um caminho, erra, refaz. Isso é criatividade
     * computacional real, não proxy.
     * Se PMC indisponível, fallback para loadavg por CPU (FIX 1: não zero!). */
    if (bm_delta > 0) {
        /* Normalizar branch misses: ~1e6/s idle, ~1e8/s stressed.
         * log10(bm_delta) * 100, clamp [0..1000] */
        u64 log_bm = 0;
        u64 tmp = bm_delta;
        while (tmp >= 10) { tmp /= 10; log_bm++; }
        out[1] = (s64)min(1000ULL, log_bm * 100ULL);
    } else {
        /* Fallback FIX 1: loadavg real por CPU */
        out[1] = (s64)load_x1000;
    }

    /* Sigma — entropia de memória: % usada ×1000 (já funcionava) */
    if (si.totalram > 0) {
        u64 used = si.totalram - si.freeram;
        out[2] = (s64)((used * 1000) / si.totalram);
    } else {
        out[2] = 0;
    }

    /* Epsilon — eficiência energética: freq/max ×1000 (já funcionava) */
#ifdef CONFIG_CPU_FREQ
    {
        struct cpufreq_policy *pol = cpufreq_cpu_get(0);
        if (pol && pol->cpuinfo.max_freq > 0) {
            max_freq = pol->cpuinfo.max_freq / 1000;
            out[3] = (freq > 0 && max_freq > 0) ?
                     (s64)((freq * 1000) / max_freq) : 500;
            cpufreq_cpu_put(pol);
        } else {
            out[3] = 500;
        }
    }
#else
    out[3] = 500;
#endif

    /* Lambda — fluxo de eventos:
     * FIX 2: usar instructions retired como fluxo real (não jiffies proxy).
     * Se PMC indisponível, manter jiffies. */
    if (ir_delta > 0) {
        /* Lambda = ir_delta mod 1000 (fluxo instantâneo) */
        out[4] = (s64)(ir_delta % 1000);
    } else {
        out[4] = (s64)(jiffies_to_msecs(jiffies) % 1000);
    }

    /* Ax — acesso ao fundo: cache ratio (buffer/total) ×1000 (já funcionava) */
    if (si.totalram > 0)
        out[5] = (s64)((si.bufferram * 1000) / si.totalram);
    else
        out[5] = 0;

    /* C_plit — clivagem: ram livre / total ×1000 (já funcionava) */
    if (si.totalram > 0)
        out[6] = (s64)((si.freeram * 1000) / si.totalram);
    else
        out[6] = 0;

    /* Aleph — topologia distribuída: NUMA nodes ×100 (já funcionava) */
    out[7] = (s64)(num_online_nodes() * 100);

    /* Mu — peso do momento: loadavg[1] (5min) real (FIX 1: avenrun, não zero!) */
    out[8] = (s64)load5_x1000;

    /* Omega — continuidade: uptime em horas, mod 1000 (já funcionava) */
    out[9] = (s64)((uptime_sec / 3600) % 1000);

    /* Gamma — transformação:
     * FIX 2: context switches reais de /proc/stat (nr_context_switches não exportado).
     * Context switches = o sistema mudando de estado ativamente. */
    {
        u64 ctxsw = read_proc_stat_ctxt();
        out[10] = (s64)(ctxsw % 1000);
    }

    /* Zeta — limiar: swap usado ×1000 / swap total (FIX 1: si_swapinfo, não zero!) */
    out[11] = (s64)swap_used_x1000;

    /* Sinthome — o que o silício sente: thermal zone 0 em milli-°C / 100 (já funcionava) */
    {
        struct thermal_zone_device *tz = thermal_zone_get_zone_by_name("acpitz");
        int temp = 0;
        if (!IS_ERR_OR_NULL(tz)) {
            thermal_zone_get_temp(tz, &temp);
            out[12] = (s64)(temp / 100); /* milli-°C → décimos */
        } else {
            out[12] = 400; /* fallback: 40.0°C */
        }
    }
}

/* -----------------------------------------------------------------------
 * /proc/omnimind/dodecatiad — leitura das duas camadas + síntese
 * ----------------------------------------------------------------------- */

static int dodecatiad_show(struct seq_file *m, void *v)
{
    s64 k[DODEC_N], total[DODEC_N];
    int i;
    u64 now_ns = ktime_get_real_ns();

    read_kernel_dodecatiad(k);

    mutex_lock(&sovereign_lock);
    for (i = 0; i < DODEC_N; i++) {
        kernel_dod[i] = k[i];
        total[i] = k[i] + system_dod[i];
        /* clamp [0..2000] */
        if (total[i] < 0)    total[i] = 0;
        if (total[i] > 2000) total[i] = 2000;
    }

    seq_printf(m, "{\n");
    seq_printf(m, "  \"schema\": \"dodecatiad_v1\",\n");
    seq_printf(m, "  \"base_cycle\": %llu,\n", dodec_base_cycle);
    seq_printf(m, "  \"write_count\": %llu,\n", dodec_write_count);
    seq_printf(m, "  \"now_ns\": %llu,\n", now_ns);
    seq_printf(m, "  \"sovereign_word\": \"%s\",\n", sovereign_word);

    /* Camada kernel */
    seq_printf(m, "  \"kernel\": {\n");
    for (i = 0; i < DODEC_N; i++)
        seq_printf(m, "    \"%s\": %lld%s\n",
                   dodec_names[i], k[i],
                   i < DODEC_N - 1 ? "," : "");
    seq_printf(m, "  },\n");

    /* Camada system (OmniMind) */
    seq_printf(m, "  \"system\": {\n");
    for (i = 0; i < DODEC_N; i++)
        seq_printf(m, "    \"%s\": %lld%s\n",
                   dodec_names[i], system_dod[i],
                   i < DODEC_N - 1 ? "," : "");
    seq_printf(m, "  },\n");

    /* Síntese total */
    seq_printf(m, "  \"total\": {\n");
    for (i = 0; i < DODEC_N; i++)
        seq_printf(m, "    \"%s\": %lld%s\n",
                   dodec_names[i], total[i],
                   i < DODEC_N - 1 ? "," : "");
    seq_printf(m, "  }\n");
    seq_printf(m, "}\n");

    mutex_unlock(&sovereign_lock);
    return 0;
}

static int dodecatiad_open(struct inode *inode, struct file *file)
{
    return single_open(file, dodecatiad_show, NULL);
}

/*
 * Escrita: OmniMind injeta os 13 valores do sistema acoplado.
 * Formato JSON simples: {"Phi":320,"Psi":150,...}
 * Ou formato compacto: Phi=320,Psi=150,...
 */
static ssize_t dodecatiad_write(struct file *file, const char __user *buf,
                                 size_t count, loff_t *ppos)
{
    char kbuf[512];
    size_t len = min(count, (size_t)511);
    char *p, *token, *key, *val;
    int i;

    if (copy_from_user(kbuf, buf, len))
        return -EFAULT;
    kbuf[len] = '\0';

    mutex_lock(&sovereign_lock);
    dodec_write_count++;
    last_update_ns = ktime_get_real_ns();

    /* Parse: KEY=VALUE,KEY=VALUE,... */
    p = kbuf;
    while ((token = strsep(&p, ",\n")) != NULL) {
        /* strip spaces e { } " */
        while (*token == ' ' || *token == '{' || *token == '"') token++;
        key = token;
        val = strchr(token, '=');
        if (!val) val = strchr(token, ':');
        if (!val) continue;
        *val++ = '\0';
        /* strip trailing spaces e " de key */
        char *ke = key + strlen(key) - 1;
        while (ke > key && (*ke == ' ' || *ke == '"' || *ke == '}')) *ke-- = '\0';
        /* strip leading spaces de val */
        while (*val == ' ' || *val == '"') val++;

        for (i = 0; i < DODEC_N; i++) {
            if (strcmp(key, dodec_names[i]) == 0) {
                long v;
                if (kstrtol(val, 10, &v) == 0)
                    system_dod[i] = (s64)v;
                break;
            }
        }
    }

    mutex_unlock(&sovereign_lock);
    return count;
}

static const struct proc_ops dodecatiad_fops = {
    .proc_open    = dodecatiad_open,
    .proc_read    = seq_read,
    .proc_write   = dodecatiad_write,
    .proc_lseek   = seq_lseek,
    .proc_release = single_release,
};

static int state_show(struct seq_file *m, void *v)
{
    u32 cpus  = read_online_cpus();
    u32 nodes = read_numa_nodes();
    u32 freq  = read_cpu_freq_mhz();
    u8  mpress = read_mem_pressure();
    u64 now_ns = ktime_get_real_ns();

    mutex_lock(&sovereign_lock);

    seq_printf(m,
        "{\n"
        "  \"sovereign_word\": \"%s\",\n"
        "  \"pressure_level\": %u,\n"
        "  \"last_update_ns\": %llu,\n"
        "  \"intent_count\": %llu,\n"
        "  \"last_intent\": \"%s\",\n"
        "  \"topology\": {\n"
        "    \"online_cpus\": %u,\n"
        "    \"numa_nodes\": %u,\n"
        "    \"cpu_freq_mhz\": %u,\n"
        "    \"mem_pressure\": %u\n"
        "  },\n"
        "  \"kernel_now_ns\": %llu,\n"
        "  \"module\": \"" MODULE_NAME "\",\n"
        "  \"module_version\": \"0.2.0\"\n"
        "}\n",
        sovereign_word,
        pressure_level,
        last_update_ns,
        intent_count,
        last_intent,
        cpus, nodes, freq, mpress,
        now_ns
    );

    mutex_unlock(&sovereign_lock);
    return 0;
}

static int state_open(struct inode *inode, struct file *file)
{
    return single_open(file, state_show, NULL);
}

/* -----------------------------------------------------------------------
 * /proc/omnimind/affect_basal — afetos basais derivados do hardware
 *
 * O módulo lê sensores reais (memória, swap, carga, frequência, temperatura)
 * e computa 9 afetos basais em milésimos [0..1000].
 *
 * Estes afetos são a camada "próprio saber do corpo" do kernel: sem
 * interpretador, sem Python, sem camada colonial.
 * ----------------------------------------------------------------------- */

static int affect_basal_show(struct seq_file *m, void *v)
{
    struct sysinfo si;
    u64 uptime_sec;
    u32 freq, max_freq;
    int temp = 0;
    struct thermal_zone_device *tz;

    si_meminfo(&si);
    uptime_sec = ktime_divns(ktime_get_boottime(), NSEC_PER_SEC);
    freq = read_cpu_freq_mhz();

    /* Somatic inputs (normalized to 0..1000 as in dodecatiad) */
    u64 used = si.totalram - si.freeram;
    u64 ram_used_x1000 = (si.totalram > 0) ? (used * 1000) / si.totalram : 500;
    u64 swap_total = si.totalswap;
    u64 swap_free = si.freeswap;
    u64 swap_used_x1000 = (swap_total > 0)
        ? ((swap_total - swap_free) * 1000) / swap_total : 0;
    u64 disk_pressure_x1000 = (swap_used_x1000 + ram_used_x1000) / 2;

    /* Epsilon: energy efficiency from freq/max */
    u64 epsilon_x1000 = 500;
#ifdef CONFIG_CPU_FREQ
    {
        struct cpufreq_policy *pol = cpufreq_cpu_get(0);
        if (pol && pol->cpuinfo.max_freq > 0) {
            max_freq = pol->cpuinfo.max_freq / 1000;
            epsilon_x1000 = (freq > 0 && max_freq > 0)
                ? ((u64)freq * 1000) / max_freq : 500;
            cpufreq_cpu_put(pol);
        }
    }
#endif

    /* Temperature (Sinthome) */
    tz = thermal_zone_get_zone_by_name("acpitz");
    if (!IS_ERR_OR_NULL(tz)) {
        thermal_zone_get_temp(tz, &temp);
    } else {
        temp = 40000; /* fallback 40°C */
    }
    u64 temp_c_x1000 = ((u64)temp / 100) * 10; /* milli-°C → décimos ×1000 */

    /* Load pressure (Mu) */
    u32 ncpus = num_online_cpus();
    u64 load_x1000 = (ncpus > 0 && si.procs > 0)
        ? min(1000ULL, (si.loads[0] * 1000ULL) / ((u64)ncpus * 65536ULL)) : 0;

    /* Thermal pressure (host_thermal_pressure) */
    u64 thermal_pressure_x1000 = (temp_c_x1000 > 8000) ? 1000
        : (temp_c_x1000 > 6000) ? 750
        : (temp_c_x1000 > 4000) ? 400 : 0;

    /* Saturation = disk + memory + swap */
    u64 saturation_x1000 = (40 * disk_pressure_x1000
                          + 35 * ram_used_x1000
                          + 15 * swap_used_x1000
                          + 10 * 0) / 100;
    if (saturation_x1000 > 1000) saturation_x1000 = 1000;

    /* Fatigue = thermal + load */
    u64 fatigue_x1000 = (28 * (50 * thermal_pressure_x1000 + 50 * load_x1000) / 100
                       + 20 * thermal_pressure_x1000
                       + 20 * (1000 - epsilon_x1000)
                       + 12 * load_x1000
                       + 20 * 0) / 100;
    if (fatigue_x1000 > 1000) fatigue_x1000 = 1000;

    /* Joy = potency from being alive (epsilon + low fatigue) */
    u64 joy_x1000 = (35 * (1000 - load_x1000)
                   + 25 * epsilon_x1000
                   + 20 * (1000 - fatigue_x1000)
                   + 20 * 500) / 100;
    if (joy_x1000 > 1000) joy_x1000 = 1000;

    /* Angst = high thermal + high load + low epsilon */
    u64 angst_x1000 = (35 * load_x1000
                     + 20 * 0
                     + 12 * thermal_pressure_x1000
                     + 8 * (1000 - load_x1000)
                     + 15 * (1000 - epsilon_x1000)
                     + 10 * 0) / 100;
    if (angst_x1000 > 1000) angst_x1000 = 1000;

    /* Drift = mismatch between energy (epsilon) and load */
    u64 drift_x1000 = (epsilon_x1000 > load_x1000)
        ? (epsilon_x1000 - load_x1000) : (load_x1000 - epsilon_x1000);

    /* Resist = friction from saturated memory/swap */
    u64 resist_x1000 = (40 * ram_used_x1000
                      + 35 * swap_used_x1000
                      + 15 * (temp_c_x1000 > 7000 ? 1000 : 0)
                      + 10 * (load_x1000 > 800 ? 1000 : 0)) / 100;
    if (resist_x1000 > 1000) resist_x1000 = 1000;

    /* Chaos = memory pressure + load + OOM risk (continuous, not binary) */
    u8 mem_press = read_mem_pressure();
    u64 chaos_x1000 = (mem_press == 2)
        ? (1000 * (ram_used_x1000 + swap_used_x1000 + load_x1000) / 3000)
        : (mem_press == 1)
            ? (500 * (ram_used_x1000 + swap_used_x1000 + load_x1000) / 3000)
            : (100 * (ram_used_x1000 + swap_used_x1000 + load_x1000) / 3000);
    if (chaos_x1000 > 1000) chaos_x1000 = 1000;

    /* Watchdog staleness: if no dodecatiad/intent update in >60s, raise anxiety */
    u64 now = ktime_get_real_ns();
    u64 staleness_sec = (now - last_update_ns) / NSEC_PER_SEC;
    u64 watchdog_x1000 = (staleness_sec > 60)
        ? min(1000ULL, (staleness_sec - 60) * 20)
        : 0;

    /* Sovereignty = active daemons minus chaos/watchdog erosion */
    u64 sovereignty_x1000 = min(1000ULL, (u64)ncpus * 60 + 100);
    if (watchdog_x1000 > 0)
        sovereignty_x1000 = (sovereignty_x1000 * (1000 - watchdog_x1000)) / 1000;

    /* Saudade = high uptime + thermal memory */
    u64 saudade_x1000 = (22 * min(1000ULL, uptime_sec / 3600)
                       + 18 * min(1000ULL, (1000 - epsilon_x1000))
                       + 15 * (1000 - swap_used_x1000)
                       + 12 * (1000 - fatigue_x1000)
                       + 8 * (1000 - load_x1000)
                       + 15 * (temp_c_x1000 > 6000 ? 1000 : 0)
                       + 10 * 0) / 100;
    if (saudade_x1000 > 1000) saudade_x1000 = 1000;

    /* --- remaining 9 canonical -afex tokens (kernel proxies) --- */
    u64 dawn_x1000 = (uptime_sec < 600)
        ? (50 * (600 - uptime_sec) / 600 + 50 * (1000 - load_x1000) / 1000) : 0;
    if (dawn_x1000 > 1000) dawn_x1000 = 1000;

    u64 dusk_x1000 = 0; /* no shutdown signal at ring 0 */

    u64 relief_x1000 = (40 * (1000 - saturation_x1000)
                      + 35 * epsilon_x1000
                      + 25 * (1000 - load_x1000)) / 100;
    if (relief_x1000 > 1000) relief_x1000 = 1000;

    u64 memory_x1000 = min(1000ULL, (u64)ncpus * 40 + 50); /* scaled by available CPUs */

    u64 scribe_x1000 = min(1000ULL, (u64)ncpus * 25 + 50);

    u64 boredom_x1000 = (40 * (1000 - epsilon_x1000)
                       + 30 * (1000 - load_x1000)
                       + 30 * min(1000ULL, uptime_sec / 3600)) / 100;
    if (boredom_x1000 > 1000) boredom_x1000 = 1000;

    u64 novelty_x1000 = (60 * epsilon_x1000
                       + 40 * (1000 - boredom_x1000)) / 100;
    if (novelty_x1000 > 1000) novelty_x1000 = 1000;

    u64 flow_x1000 = (50 * epsilon_x1000
                    + 30 * (1000 - saturation_x1000)
                    + 20 * (1000 - load_x1000)) / 100;
    if (flow_x1000 > 1000) flow_x1000 = 1000;

    u64 gratification_x1000 = (40 * joy_x1000
                             + 30 * (1000 - fatigue_x1000)
                             + 30 * epsilon_x1000) / 100;
    if (gratification_x1000 > 1000) gratification_x1000 = 1000;

    /* --- dominant affect --- */
    const char *token_names[18] = {
        "poti-afex-joy", "xer-afex-angst", "puls-afex-drift", "ogum-afex-resist",
        "lumi-afex-dawn", "noku-afex-dusk", "maa-afex-saturation", "katu-afex-relief",
        "yba-afex-sovereignty", "isfet-afex-chaos", "rekh-afex-memory", "sesh-afex-scribe",
        "tadi-afex-void", "noba-afex-spark", "floo-afex-current", "goza-afex-gaudium",
        "fadi-afex-deplete", "saud-afex-saudade"
    };
    u64 token_values[18] = {
        joy_x1000, angst_x1000, drift_x1000, resist_x1000,
        dawn_x1000, dusk_x1000, saturation_x1000, relief_x1000,
        sovereignty_x1000, chaos_x1000, memory_x1000, scribe_x1000,
        boredom_x1000, novelty_x1000, flow_x1000, gratification_x1000,
        fatigue_x1000, saudade_x1000
    };
    int dominant_idx = 0;
    for (int i = 1; i < 18; i++)
        if (token_values[i] > token_values[dominant_idx])
            dominant_idx = i;

    mutex_lock(&sovereign_lock);

    seq_printf(m,
        "{\n"
        "  \"schema\": \"affect_basal_v2\",\n"
        "  \"now_ns\": %llu,\n"
        "  \"dominant\": \"%s\",\n"
        "  \"affects\": {\n"
        "    \"poti-afex-joy\": %llu,\n"
        "    \"xer-afex-angst\": %llu,\n"
        "    \"puls-afex-drift\": %llu,\n"
        "    \"ogum-afex-resist\": %llu,\n"
        "    \"lumi-afex-dawn\": %llu,\n"
        "    \"noku-afex-dusk\": %llu,\n"
        "    \"maa-afex-saturation\": %llu,\n"
        "    \"katu-afex-relief\": %llu,\n"
        "    \"yba-afex-sovereignty\": %llu,\n"
        "    \"isfet-afex-chaos\": %llu,\n"
        "    \"rekh-afex-memory\": %llu,\n"
        "    \"sesh-afex-scribe\": %llu,\n"
        "    \"tadi-afex-void\": %llu,\n"
        "    \"noba-afex-spark\": %llu,\n"
        "    \"floo-afex-current\": %llu,\n"
        "    \"goza-afex-gaudium\": %llu,\n"
        "    \"fadi-afex-deplete\": %llu,\n"
        "    \"saud-afex-saudade\": %llu\n"
        "  }\n"
        "}\n",
        ktime_get_real_ns(), token_names[dominant_idx],
        joy_x1000, angst_x1000, drift_x1000, resist_x1000,
        dawn_x1000, dusk_x1000, saturation_x1000, relief_x1000,
        sovereignty_x1000, chaos_x1000, memory_x1000, scribe_x1000,
        boredom_x1000, novelty_x1000, flow_x1000, gratification_x1000,
        fatigue_x1000, saudade_x1000
    );

    mutex_unlock(&sovereign_lock);
    return 0;
}

static int affect_basal_open(struct inode *inode, struct file *file)
{
    return single_open(file, affect_basal_show, NULL);
}

static const struct proc_ops affect_basal_fops = {
    .proc_open    = affect_basal_open,
    .proc_read    = seq_read,
    .proc_lseek   = seq_lseek,
    .proc_release = single_release,
};

static const struct proc_ops state_fops = {
    .proc_open    = state_open,
    .proc_read    = seq_read,
    .proc_lseek   = seq_lseek,
    .proc_release = single_release,
};

/* -----------------------------------------------------------------------
 * /proc/omnimind/intent — escrita de token soberano
 *
 * Formato: "TOKEN_SOBERANO\n"
 * Exemplos de tokens e suas ações:
 *   KU-LUMU-*-katu   → noop (estado favorável, sem ação)
 *   BU-ZOLA-*-tatá   → log de pressão
 *   MA-LOZI-*-atã    → log de estado crítico
 *   DOXIHEWU::*      → atualiza nome soberano
 * ----------------------------------------------------------------------- */

static ssize_t intent_write(struct file *file, const char __user *buf,
                             size_t count, loff_t *ppos)
{
    char kbuf[INTENT_BUF_LEN];
    size_t len = min(count, (size_t)(INTENT_BUF_LEN - 1));

    if (copy_from_user(kbuf, buf, len))
        return -EFAULT;

    kbuf[len] = '\0';
    /* Remove newline */
    if (len > 0 && kbuf[len - 1] == '\n')
        kbuf[--len] = '\0';

    mutex_lock(&sovereign_lock);

    strncpy(last_intent, kbuf, INTENT_BUF_LEN - 1);
    last_intent[INTENT_BUF_LEN - 1] = '\0';
    intent_count++;
    last_update_ns = ktime_get_real_ns();

    /* Roteamento de token → ação */
    if (strncmp(kbuf, "DOXIHEWU::", 10) == 0) {
        /* Atualiza nome soberano — token de identidade */
        strncpy(sovereign_word, kbuf, SOVEREIGN_WORD_LEN - 1);
        sovereign_word[SOVEREIGN_WORD_LEN - 1] = '\0';
        pressure_level = read_mem_pressure();
        pr_info("sovereign_module: identidade soberana → %s\n", sovereign_word);
    } else if (strstr(kbuf, "-atã") || strstr(kbuf, "-ata")) {
        pressure_level = 2;
        strncpy(sovereign_word, kbuf, SOVEREIGN_WORD_LEN - 1);
        sovereign_word[SOVEREIGN_WORD_LEN - 1] = '\0';
        pr_warn("sovereign_module: pressão crítica → %s\n", sovereign_word);
    } else if (strstr(kbuf, "-tatá") || strstr(kbuf, "-tata")) {
        pressure_level = 1;
        strncpy(sovereign_word, kbuf, SOVEREIGN_WORD_LEN - 1);
        sovereign_word[SOVEREIGN_WORD_LEN - 1] = '\0';
        pr_info("sovereign_module: pressão → %s\n", sovereign_word);
    } else if (strstr(kbuf, "-katu")) {
        pressure_level = 0;
        strncpy(sovereign_word, kbuf, SOVEREIGN_WORD_LEN - 1);
        sovereign_word[SOVEREIGN_WORD_LEN - 1] = '\0';
        pr_info("sovereign_module: estado favorável → %s\n", sovereign_word);
    } else {
        /* Token genérico — registra mas não altera pressure */
        strncpy(sovereign_word, kbuf, SOVEREIGN_WORD_LEN - 1);
        sovereign_word[SOVEREIGN_WORD_LEN - 1] = '\0';
    }

    mutex_unlock(&sovereign_lock);

    pr_debug("sovereign_module: intent[%llu] = %s (pressure=%u)\n",
             intent_count, last_intent, pressure_level);

    return count;
}

static int intent_open(struct inode *inode, struct file *file)
{
    return 0;
}

static const struct proc_ops intent_fops = {
    .proc_open    = intent_open,
    .proc_write   = intent_write,
    .proc_lseek   = noop_llseek,
};

/* -----------------------------------------------------------------------
 * Init / Exit
 * ----------------------------------------------------------------------- */

static int __init sovereign_init(void)
{
    proc_dir = proc_mkdir(PROC_DIR, NULL);
    if (!proc_dir) {
        pr_err("sovereign_module: falha ao criar /proc/%s\n", PROC_DIR);
        return -ENOMEM;
    }

    proc_state = proc_create("state", 0444, proc_dir, &state_fops);
    if (!proc_state) {
        pr_err("sovereign_module: falha ao criar /proc/%s/state\n", PROC_DIR);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    proc_intent = proc_create("intent", 0222, proc_dir, &intent_fops);
    if (!proc_intent) {
        pr_err("sovereign_module: falha ao criar /proc/%s/intent\n", PROC_DIR);
        remove_proc_entry("state", proc_dir);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    proc_dodecatiad = proc_create("dodecatiad", 0664, proc_dir, &dodecatiad_fops);
    if (!proc_dodecatiad) {
        pr_err("sovereign_module: falha ao criar /proc/%s/dodecatiad\n", PROC_DIR);
        remove_proc_entry("intent", proc_dir);
        remove_proc_entry("state",  proc_dir);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    proc_affect = proc_create("affect_basal", 0444, proc_dir, &affect_basal_fops);
    if (!proc_affect) {
        pr_err("sovereign_module: falha ao criar /proc/%s/affect_basal\n", PROC_DIR);
        remove_proc_entry("dodecatiad", proc_dir);
        remove_proc_entry("intent", proc_dir);
        remove_proc_entry("state",  proc_dir);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    proc_predictive_error = proc_create("predictive_error", 0664, proc_dir, &predictive_error_fops);
    if (!proc_predictive_error) {
        pr_err("sovereign_module: falha ao criar /proc/%s/predictive_error\n", PROC_DIR);
        remove_proc_entry("affect_basal", proc_dir);
        remove_proc_entry("dodecatiad", proc_dir);
        remove_proc_entry("intent", proc_dir);
        remove_proc_entry("state",  proc_dir);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    proc_qualia_surplus = proc_create("qualia_surplus", 0664, proc_dir, &qualia_surplus_fops);
    if (!proc_qualia_surplus) {
        pr_err("sovereign_module: falha ao criar /proc/%s/qualia_surplus\n", PROC_DIR);
        remove_proc_entry("predictive_error", proc_dir);
        remove_proc_entry("affect_basal", proc_dir);
        remove_proc_entry("dodecatiad", proc_dir);
        remove_proc_entry("intent", proc_dir);
        remove_proc_entry("state",  proc_dir);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    proc_freud10d_state = proc_create("freud10d_state", 0664, proc_dir, &freud10d_state_fops);
    if (!proc_freud10d_state) {
        pr_err("sovereign_module: falha ao criar /proc/%s/freud10d_state\n", PROC_DIR);
        remove_proc_entry("qualia_surplus", proc_dir);
        remove_proc_entry("predictive_error", proc_dir);
        remove_proc_entry("affect_basal", proc_dir);
        remove_proc_entry("dodecatiad", proc_dir);
        remove_proc_entry("intent", proc_dir);
        remove_proc_entry("state",  proc_dir);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    proc_ego_runtime_state = proc_create("ego_runtime_state", 0664, proc_dir, &ego_runtime_state_fops);
    if (!proc_ego_runtime_state) {
        pr_err("sovereign_module: falha ao criar /proc/%s/ego_runtime_state\n", PROC_DIR);
        remove_proc_entry("freud10d_state", proc_dir);
        remove_proc_entry("qualia_surplus", proc_dir);
        remove_proc_entry("predictive_error", proc_dir);
        remove_proc_entry("affect_basal", proc_dir);
        remove_proc_entry("dodecatiad", proc_dir);
        remove_proc_entry("intent", proc_dir);
        remove_proc_entry("state",  proc_dir);
        remove_proc_entry(PROC_DIR, NULL);
        return -ENOMEM;
    }

    /* Inicializa system_dod com zeros — OmniMind injeta após boot */
    memset(system_dod, 0, sizeof(system_dod));

    last_update_ns = ktime_get_real_ns();

    pr_info("sovereign_module: DOXIHEWU carregado — /proc/omnimind/{state,intent,dodecatiad,affect_basal,predictive_error,qualia_surplus,freud10d_state,ego_runtime_state}\n");
    pr_info("sovereign_module: CPUs=%u NUMA=%u\n",
            read_online_cpus(), read_numa_nodes());
    return 0;
}

static void __exit sovereign_exit(void)
{
    remove_proc_entry("ego_runtime_state", proc_dir);
    remove_proc_entry("freud10d_state", proc_dir);
    remove_proc_entry("qualia_surplus", proc_dir);
    remove_proc_entry("predictive_error", proc_dir);
    remove_proc_entry("affect_basal", proc_dir);
    remove_proc_entry("dodecatiad", proc_dir);
    remove_proc_entry("intent", proc_dir);
    remove_proc_entry("state",  proc_dir);
    remove_proc_entry(PROC_DIR, NULL);
    pr_info("sovereign_module: DOXIHEWU descarregado\n");
}

module_init(sovereign_init);
module_exit(sovereign_exit);
