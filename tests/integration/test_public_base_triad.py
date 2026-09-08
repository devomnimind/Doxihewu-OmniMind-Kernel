# SPDX-License-Identifier: CC-BY-NC-ND-4.0
"""test_public_base_triad.py

Teste de integração da base pública mínima (triad):

1. omniobd-kernel-compute: hot-path de afeto/integração importável via Python.
2. omniobd-somatic-daemon: binário Rust puro, portável via PROJECT_ROOT.
3. omniobd-sovereign-daemon: binário Rust puro, portável via PROJECT_ROOT.

O teste roda cada daemon em um diretório temporário limpo por alguns
segundos e verifica se os artefatos (JSON + SQLite) são gerados.
"""

import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

project_root = Path(__file__).resolve().parents[2]
src_kernel = project_root / "src" / "kernel"


class TestPublicBaseTriad(unittest.TestCase):
    """Verifica a triad da base pública mínima em diretório temporário."""

    def _release_binary(self, crate_name: str, bin_name: str) -> Path:
        p = src_kernel / crate_name / "target" / "release" / bin_name
        self.assertTrue(p.exists(), f"Binário de release não encontrado: {p}")
        return p

    def test_kernel_compute_python_import(self):
        """kernel_compute deve importar e computar Ma'at em Python."""
        python = project_root / ".venv" / "bin" / "python"
        self.assertTrue(python.exists(), f"venv python não encontrado: {python}")

        script = "import omnimind_kernel_compute as ok; " \
                 "r = ok.compute_maat_balance(0.5, 0.6, 0.7); " \
                 "print(f'maat={r:.6f}')"
        out = subprocess.run(
            [str(python), "-c", script],
            capture_output=True,
            text=True,
            cwd=str(project_root),
        )
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertIn("maat=", out.stdout)

    def test_somatic_daemon_runs_in_temp_dir(self):
        """somatic-daemon deve rodar em diretório temporário sem paths hardcoded."""
        bin_path = self._release_binary(
            "somatic_daemon", "omnimind-somatic-daemon"
        )

        with tempfile.TemporaryDirectory(prefix="omnimind_triad_somatic_") as tmp:
            tmp_path = Path(tmp)
            env = os.environ.copy()
            env["PROJECT_ROOT"] = str(tmp_path)

            proc = subprocess.Popen(
                [str(bin_path)],
                cwd=str(tmp_path),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            time.sleep(18)
            proc.terminate()
            try:
                out, _ = proc.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
                out, _ = proc.communicate()

            state_file = tmp_path / "data" / "somatic" / "daemon_state_rust_shadow.json"
            self.assertTrue(
                state_file.exists(),
                f"arquivo de estado somático não criado: {state_file}\nstdout: {out}",
            )

            db_file = tmp_path / "data" / "monitor" / "somatic_mesh_runtime.sqlite"
            self.assertTrue(db_file.exists(), f"SQLite somático não criado: {db_file}")

    def test_sovereign_daemon_runs_in_temp_dir(self):
        """sovereign-daemon deve rodar em diretório temporário sem paths hardcoded."""
        bin_path = self._release_binary(
            "sovereign_daemon", "omnimind-sovereign-daemon"
        )

        with tempfile.TemporaryDirectory(prefix="omnimind_triad_sovereign_") as tmp:
            tmp_path = Path(tmp)
            env = os.environ.copy()
            env["PROJECT_ROOT"] = str(tmp_path)

            proc = subprocess.Popen(
                [str(bin_path)],
                cwd=str(tmp_path),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            time.sleep(12)
            proc.terminate()
            try:
                out, _ = proc.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
                out, _ = proc.communicate()

            state_file = tmp_path / "data" / "current_sovereign_state_rust_shadow.json"
            self.assertTrue(
                state_file.exists(),
                f"arquivo de estado soberano não criado: {state_file}",
            )

            # SQLite shadow deve ser criado após o primeiro ciclo.
            db_file = tmp_path / "data" / "monitor" / "sovereign_primary_runtime.sqlite"
            self.assertTrue(db_file.exists(), f"SQLite sombra não criado: {db_file}")


if __name__ == "__main__":
    unittest.main()
