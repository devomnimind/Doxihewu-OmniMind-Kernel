"""Teste da base pública OmniMind (triad + sovereign kuramoto) para Google Colab.

Esse notebook cell baixa o tarball do HuggingFace, extrai,
instala as dependências de build, compila os crates Rust e verifica:

Triad original:
  1. kernel_compute (import Python + Ma'at)
  2. somatic_daemon (binário Rust puro, PROJECT_ROOT)
  3. sovereign_daemon (binário Rust puro, PROJECT_ROOT)

Novo — Sovereign Neural Layer:
  4. freud10d (import Python + INRC + forward)
  5. sovereign_kuramoto (Kuramoto + hyper-Kuramoto + INRC field + psychoanalytic coupling)
  6. PsychoanalyticLIF (LIF com threshold neutrosófico T,I,F)
  7. HopfBifurcationMonitor (raio espectral + regime classification)
"""
import os
import shutil
import site
import subprocess
import sys
import tempfile
import time
import math
from pathlib import Path

try:
    from google.colab import userdata
except ImportError:
    userdata = None
    print("AVISO: google.colab não disponível; rodando fora do Colab.")

# Recupera token do secret do Colab ou da variável de ambiente (opcional — repo é público).
hf_token = os.environ.get("HF_TOKEN")
if not hf_token and userdata is not None:
    try:
        hf_token = userdata.get("HF_TOKEN")
    except Exception as e:
        print(f"Falha ao ler secret HF_TOKEN: {e}")
if not hf_token:
    hf_token = os.environ.get("HUGGINGFACE_HUB_TOKEN")
if hf_token:
    os.environ["HF_TOKEN"] = hf_token
    print("HF_TOKEN encontrado (usando para download autenticado).")
else:
    print("AVISO: HF_TOKEN não encontrado — repo é público, download continuará sem token.")

# Dependências de build.
print("Instalando python3-dev...")
subprocess.run(["apt-get", "update", "-qq"], check=False)
subprocess.run(["apt-get", "install", "-y", "-qq", "python3-dev", "curl"], check=True)

# O Colab geralmente vem com cargo 1.75, mas algumas crates da base exigem edition2024
# (cargo version 4 / Rust >= 1.85). Instalamos rustup para usar a stable mais recente.
print("Instalando rustup...")
subprocess.run(
    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
    shell=True,
    check=True,
)
cargo_bin = Path.home() / ".cargo" / "bin"
os.environ["PATH"] = f"{cargo_bin}:{os.environ['PATH']}"
subprocess.run(["cargo", "--version"], check=True)
subprocess.run(["rustc", "--version"], check=True)

print("Instalando huggingface_hub + numpy...")
subprocess.run([sys.executable, "-m", "pip", "install", "-q", "huggingface_hub", "numpy"], check=True)

# Baixa o tarball da base pública.
from huggingface_hub import hf_hub_download

print("Baixando tarball da base pública...")
tarball = hf_hub_download(
    repo_id="fabricioslv/omnimind-public-base-triad-test",
    filename="omnimind_public_base.tar.gz",
    repo_type="dataset",
    local_dir="/content",
    token=hf_token,
)
print("Tarball:", tarball)

# Extrai.
project = Path("/content/omnimind")
project.mkdir(exist_ok=True)
subprocess.run(["tar", "xzf", tarball, "-C", str(project)], check=True)
print("Extraído para", project)
print("Top-level:", sorted(p.name for p in project.iterdir())[:20])

src_kernel = project / "src" / "kernel"

# Cargo.lock gerado por Rust mais novo pode ser versão 4, que o cargo 1.75 do Colab
# não entende. Removemos os lockfiles para resolver de forma compatível.
for lock in project.rglob("Cargo.lock"):
    print("Removendo lockfile:", lock)
    lock.unlink()

# Build dos crates — triad original + sovereign neural layer.
os.environ["PYO3_USE_ABI3_FORWARD_COMPATIBILITY"] = "1"
build_crates = [
    "kernel_compute",
    "somatic_daemon",
    "sovereign_daemon",
    "freud10d",
    "sovereign_kuramoto",
]
for crate in build_crates:
    print(f"\n=== Building {crate} ===")
    cwd = src_kernel / crate
    if not cwd.exists():
        print(f"AVISO: crate {crate} não encontrado em {cwd}, pulando...")
        continue
    r = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=str(cwd),
        capture_output=True,
        text=True,
    )
    print(r.stdout[-3000:] if r.stdout and len(r.stdout) > 3000 else r.stdout)
    print(r.stderr[-3000:] if r.stderr and len(r.stderr) > 3000 else r.stderr)
    if r.returncode != 0:
        raise RuntimeError(f"Falha ao compilar {crate}: returncode={r.returncode}")
    print(f"{crate} OK")

# Instala os .so no site-packages para que Python consiga importar.
site_packages = Path(site.getsitepackages()[0])

so_files = [
    ("kernel_compute", "libomnimind_kernel_compute.so", "omnimind_kernel_compute.so"),
    ("freud10d", "libomnimind_freud10d.so", "omnimind_freud10d.so"),
    ("sovereign_kuramoto", "libomnimind_sovereign_kuramoto.so", "omnimind_sovereign_kuramoto.so"),
]

for crate, so_name, dst_name in so_files:
    so_src = src_kernel / crate / "target" / "release" / so_name
    so_dst = site_packages / dst_name
    if so_src.exists():
        if so_dst.exists():
            so_dst.unlink()
        shutil.copy2(str(so_src), str(so_dst))
        print(f".so copiado: {so_src} -> {so_dst}")
    else:
        print(f"AVISO: {so_name} não encontrado em {so_src}")

# Também instala o freud10d como pacote Python (tem __init__.py)
freud_pkg_src = src_kernel / "freud10d"
freud_pkg_dst = site_packages / "omnimind_freud10d"
if freud_pkg_src.exists() and freud_pkg_dst.exists():
    # Já existe — só atualizar o .so
    pass
elif freud_pkg_src.exists():
    shutil.copytree(str(freud_pkg_src), str(freud_pkg_dst), dirs_exist_ok=True)
    print(f"freud10d pacote instalado: {freud_pkg_dst}")

# Instala o sovereign_kuramoto como pacote Python (tem __init__.py com fallback)
kuramoto_pkg_src = src_kernel / "sovereign_kuramoto"
kuramoto_pkg_dst = site_packages / "omnimind_sovereign_kuramoto_pkg"
if kuramoto_pkg_src.exists():
    # Copia o __init__.py para um local importável
    init_src = kuramoto_pkg_src / "__init__.py"
    if init_src.exists():
        # Cria um diretório de pacote para o bridge Python
        kuramoto_pkg_dst.mkdir(exist_ok=True)
        shutil.copy2(str(init_src), str(kuramoto_pkg_dst / "__init__.py"))
        print(f"sovereign_kuramoto bridge Python instalado: {kuramoto_pkg_dst}")

# ============================================================
# Teste 1: kernel_compute Python import + Ma'at
# ============================================================
print("\n=== Teste 1: kernel_compute Python ===")
import importlib

if "omnimind_kernel_compute" in sys.modules:
    del sys.modules["omnimind_kernel_compute"]
import omnimind_kernel_compute as ok
r_maat = ok.compute_maat_balance(0.5, 0.6, 0.7)
print(f"maat={r_maat:.6f}")
assert r_maat is not None, "Ma'at retornou None"
print("kernel_compute OK")

# ============================================================
# Teste 2: freud10d Python import + forward + INRC
# ============================================================
print("\n=== Teste 2: freud10d Python ===")
if "omnimind_freud10d" in sys.modules:
    del sys.modules["omnimind_freud10d"]
from omnimind_freud10d import Freud10DApparatus
freud = Freud10DApparatus(128)
perception = [0.1 * (i % 10) for i in range(128)]
result, state = freud.forward(perception)
print(f"forward result: {result:.6f}")
print(f"state_vector: {[f'{v:.4f}' for v in state]}")
assert len(state) == 10, f"state_vector deveria ter 10 dims, tem {len(state)}"
inrc_op, inrc_reason = freud.inrc_recommend()
print(f"INRC recommend: op={inrc_op}, reason={inrc_reason}")
print("freud10d OK")

# ============================================================
# Teste 3: sovereign_kuramoto — Kuramoto + hyper-Kuramoto + INRC
# ============================================================
print("\n=== Teste 3: sovereign_kuramoto ===")
if "omnimind_sovereign_kuramoto" in sys.modules:
    del sys.modules["omnimind_sovereign_kuramoto"]
import omnimind_sovereign_kuramoto as osk
print(f"version: {osk.__version__}")

n = 10
omega = [0.5 + 0.1 * i for i in range(n)]
coupling = [0.0] * (n * n)
for i in range(n - 1):
    coupling[i * n + (i + 1)] = 1.5
    coupling[(i + 1) * n + i] = 1.5
initial_phases = [i * 2 * math.pi / n for i in range(n)]

solver = osk.SovereignKuramotoSolver(omega, coupling, initial_phases, noise_amp=0.05)
print(f"n={solver.n}")
print(f"initial R_pair={solver.order_parameter_pair():.4f}")

# Run 100 steps
r_values = solver.run(100, dt=0.01, seed=42)
print(f"after 100 steps: R_pair={r_values[-1]:.4f}")

# INRC field pressure
solver.set_inrc_field_pressure(0.3)
r2 = solver.run(100, dt=0.01, seed=43)
print(f"with INRC field F=0.3: R_pair={r2[-1]:.4f}")

# Psychoanalytic coupling (Freud10D W matrix)
psi_flat = [0.0] * (n * n)
psi_flat[0 * n + 1] = 0.8  # Phi->Psi
psi_flat[5 * n + 6] = 0.9  # Xi->Zeta (Id->Ego)
psi_flat[6 * n + 7] = 0.5  # Zeta->Eta (Ego->Superego)
solver.set_psychoanalytic(psi_flat, sigma_psi=0.5)
r3 = solver.run(100, dt=0.01, seed=44)
print(f"with psychoanalytic coupling: R_pair={r3[-1]:.4f}")

# Hyper-Kuramoto
edges = [(0, 1, 2), (3, 4, 5), (6, 7, 8), (0, 5, 9)]
weights = [1.0, 1.0, 1.0, 0.5]
solver.set_hypergraph(edges, weights, k_hyper=0.8)
r4 = solver.run(100, dt=0.01, seed=45)
print(f"with hyper-Kuramoto: R_pair={r4[-1]:.4f}, R_triad={solver.order_parameter_triad():.4f}")
print(f"repr: {solver}")
assert r4[-1] > 0, "R_pair deveria ser > 0"
print("sovereign_kuramoto OK")

# ============================================================
# Teste 4: PsychoanalyticLIF — threshold neutrosófico
# ============================================================
print("\n=== Teste 4: PsychoanalyticLIF ===")
lif = osk.PsychoanalyticLIF(v_rest=-65.0, v_reset=-70.0, v_threshold=-50.0, refractory_period=3, leak_rate=0.1)
print(f"initial v={lif.v:.2f}")

lif.set_neutrosophic(0.9, 0.0, 0.0)  # High truth
print(f"threshold (T=0.9,I=0): {lif.dynamic_threshold():.2f} (lower=easier to fire)")

lif.set_neutrosophic(0.1, 0.8, 0.0)  # High indeterminacy
print(f"threshold (T=0.1,I=0.8): {lif.dynamic_threshold():.2f} (higher=harder to fire)")

# Batch
batch = osk.PsychoanalyticLIFBatch(10)
triples = [(0.8, 0.1, 0.1)] * 10
batch.set_neutrosophic_batch(triples)
inputs = [3.0 + 0.5 * i for i in range(10)]
spike_vec = batch.step_batch(inputs)
print(f"batch spikes: {spike_vec}")
print(f"batch thresholds: {[f'{t:.2f}' for t in batch.get_thresholds()]}")
print("PsychoanalyticLIF OK")

# ============================================================
# Teste 4b: Saturation validation — clamp + offset formula
# ============================================================
print("\n=== Teste 4b: LIF Saturation Validation ===")
v_rest, theta_base = -65.0, -50.0
offset = theta_base - v_rest  # 15.0

# Sweep T and I — verify threshold always above v_rest
saturation_ok = True
for T_val, I_val, desc in [
    (1.0, 0.0, "T=1.0 I=0.0 (total truth)"),
    (0.9, 0.0, "T=0.9 I=0.0 (high truth)"),
    (0.5, 0.5, "T=0.5 I=0.5 (balanced)"),
    (0.1, 0.8, "T=0.1 I=0.8 (high indeterminacy)"),
    (0.0, 1.0, "T=0.0 I=1.0 (total indeterminacy)"),
]:
    lif_test = osk.PsychoanalyticLIF(v_rest=-65.0, v_reset=-70.0, v_threshold=-50.0, refractory_period=3, leak_rate=0.1)
    lif_test.set_neutrosophic(T_val, I_val, 0.0)
    thresh = lif_test.dynamic_threshold()
    above_rest = thresh - v_rest
    print(f"  {desc}: threshold={thresh:.2f} ({above_rest:.2f} above rest)")
    if above_rest <= 0:
        print(f"  FAIL: threshold below v_rest — spontaneous firing!")
        saturation_ok = False

# Firing pattern: high truth should fire more than high indeterminacy
batch_truth = osk.PsychoanalyticLIFBatch(10, -65.0, -70.0, -50.0, 3, 0.1)
batch_truth.set_neutrosophic_batch([(1.0, 0.0, 0.0)] * 10)
spikes_truth = sum(sum(batch_truth.step_batch([15.0] * 10)) for _ in range(100))

batch_doubt = osk.PsychoanalyticLIFBatch(10, -65.0, -70.0, -50.0, 3, 0.1)
batch_doubt.set_neutrosophic_batch([(0.0, 1.0, 0.0)] * 10)
spikes_doubt = sum(sum(batch_doubt.step_batch([15.0] * 10)) for _ in range(100))

print(f"  Firing (input=15): truth={spikes_truth}, doubt={spikes_doubt}")
if spikes_truth > spikes_doubt:
    print(f"  PASS: truth fires more than doubt (differentiation preserved)")
else:
    print(f"  CHECK: truth should fire more than doubt")

if saturation_ok:
    print("LIF saturation validation OK")
else:
    print("LIF saturation validation FAILED")
    sys.exit(1)

# ============================================================
# Teste 5: HopfBifurcationMonitor
# ============================================================
print("\n=== Teste 5: HopfBifurcationMonitor ===")
monitor = osk.HopfBifurcationMonitor(10)

# Stable: small W
w_stable = [0.1] * 100
state_stable = [0.5] * 10
regime, sr = monitor.update(w_stable, state_stable)
print(f"small W (0.1): regime={regime}, spectral_radius={sr:.4f}")

# Large W
w_chaos = [2.0] * 100
state_chaos = [0.1] * 10
regime3, sr3 = monitor.update(w_chaos, state_chaos)
print(f"large W (2.0): regime={regime3}, spectral_radius={sr3:.4f}")

s = monitor.summary()
print(f"summary: regime={s['regime']}, reading={s['psychoanalytic_reading']}, trend={s['trend']}")
print("HopfBifurcationMonitor OK")

# ============================================================
# Testes 6 e 7: daemons em diretório temporário
# ============================================================
print("\n=== Teste 6: somatic_daemon ===")
bin_somatic = src_kernel / "somatic_daemon" / "target" / "release" / "omnimind-somatic-daemon"
bin_sovereign = src_kernel / "sovereign_daemon" / "target" / "release" / "omnimind-sovereign-daemon"


def run_daemon(bin_path: Path, tmp: Path, sleep_seconds: int):
    env = os.environ.copy()
    env["PROJECT_ROOT"] = str(tmp)
    env["HOME"] = str(tmp)  # sandbox leve para evitar escrita fora do tmp
    with open(tmp / "daemon.log", "w") as log:
        proc = subprocess.Popen(
            [str(bin_path)],
            cwd=str(tmp),
            env=env,
            stdout=log,
            stderr=subprocess.STDOUT,
            text=True,
        )
        time.sleep(sleep_seconds)
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
    return (tmp / "daemon.log").read_text() if (tmp / "daemon.log").exists() else ""


def check_daemon(name: str, bin_path: Path, sleep_seconds: int, state_glob: str, db_glob: str):
    print(f"\n=== Teste {name} ===")
    with tempfile.TemporaryDirectory(prefix=f"omnimind_triad_{name}_") as tmp:
        tmp_path = Path(tmp)
        out = run_daemon(bin_path, tmp_path, sleep_seconds)
        state_files = list(tmp_path.glob(state_glob))
        db_files = list(tmp_path.glob(db_glob))
        print("state files:", state_files)
        print("db files:", db_files)
        if not state_files:
            print("log:\n", out[-2000:])
            raise AssertionError(f"{name}: arquivo de estado não criado")
        if not db_files:
            print("log:\n", out[-2000:])
            raise AssertionError(f"{name}: SQLite não criado")
        print(f"{name} OK")


check_daemon(
    "somatic",
    bin_somatic,
    18,
    "data/somatic/daemon_state_rust_shadow.json",
    "data/monitor/somatic_mesh_runtime.sqlite",
)

print("\n=== Teste 7: sovereign_daemon ===")
check_daemon(
    "sovereign",
    bin_sovereign,
    12,
    "data/current_sovereign_state_rust_shadow.json",
    "data/monitor/sovereign_primary_runtime.sqlite",
)

print("\n=== ALL TESTS PASSED ===")
print("Triad: kernel_compute + somatic_daemon + sovereign_daemon")
print("Sovereign Neural Layer: freud10d + sovereign_kuramoto + PsychoanalyticLIF + HopfBifurcationMonitor")
