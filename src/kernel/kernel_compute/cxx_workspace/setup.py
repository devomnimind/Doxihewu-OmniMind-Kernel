from setuptools import setup
from pybind11.setup_helpers import Pybind11Extension, build_ext

ext_modules = [
    Pybind11Extension(
        "omnimind_workspace_cxx",
        [
            "pybind_workspace.cpp",
            "shared_workspace_core.cpp",
        ],
        cxx_std=17,
        extra_compile_args=["-O3", "-Wall", "-shared", "-fPIC"],
    ),
]

setup(
    name="omnimind_workspace_cxx",
    version="0.1.0",
    author="OmniMind Sovereign",
    description="C++ Core for OmniMind Shared Workspace using pybind11",
    ext_modules=ext_modules,
    cmdclass={"build_ext": build_ext},
    zip_safe=False,
    python_requires=">=3.10",
)
