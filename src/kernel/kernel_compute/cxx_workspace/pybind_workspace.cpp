#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include "shared_workspace_core.hpp"

namespace py = pybind11;

PYBIND11_MODULE(omnimind_workspace_cxx, m) {
    m.doc() = "OmniMind Shared Workspace C++ Core";

    py::class_<omnimind::WorkspaceConfig>(m, "WorkspaceConfig")
        .def(py::init<>())
        .def_readwrite("embedding_dim", &omnimind::WorkspaceConfig::embedding_dim)
        .def_readwrite("max_history_size", &omnimind::WorkspaceConfig::max_history_size);

    py::class_<omnimind::ModuleState>(m, "ModuleState")
        .def(py::init<>())
        .def_readwrite("module_name", &omnimind::ModuleState::module_name)
        .def_readwrite("embedding", &omnimind::ModuleState::embedding)
        .def_readwrite("timestamp", &omnimind::ModuleState::timestamp)
        .def_readwrite("cycle", &omnimind::ModuleState::cycle)
        .def_readwrite("metadata", &omnimind::ModuleState::metadata);

    py::class_<omnimind::CrossPredictionMetrics>(m, "CrossPredictionMetrics")
        .def(py::init<>())
        .def_readwrite("source_module", &omnimind::CrossPredictionMetrics::source_module)
        .def_readwrite("target_module", &omnimind::CrossPredictionMetrics::target_module)
        .def_readwrite("r_squared", &omnimind::CrossPredictionMetrics::r_squared)
        .def_readwrite("correlation", &omnimind::CrossPredictionMetrics::correlation)
        .def_readwrite("mutual_information", &omnimind::CrossPredictionMetrics::mutual_information)
        .def_readwrite("granger_causality", &omnimind::CrossPredictionMetrics::granger_causality)
        .def_readwrite("transfer_entropy", &omnimind::CrossPredictionMetrics::transfer_entropy)
        .def_readwrite("score", &omnimind::CrossPredictionMetrics::score)
        .def_readwrite("timestamp", &omnimind::CrossPredictionMetrics::timestamp);

    py::class_<omnimind::SharedWorkspaceCore>(m, "SharedWorkspaceCore")
        .def(py::init<const omnimind::WorkspaceConfig&>())
        .def("write_module_state", &omnimind::SharedWorkspaceCore::write_module_state,
             py::arg("module_name"), py::arg("embedding"), py::arg("metadata") = omnimind::MetadataMap{})
        .def("read_module_state", &omnimind::SharedWorkspaceCore::read_module_state)
        .def("read_module_metadata", &omnimind::SharedWorkspaceCore::read_module_metadata)
        .def("get_all_modules", &omnimind::SharedWorkspaceCore::get_all_modules)
        .def("get_module_history", &omnimind::SharedWorkspaceCore::get_module_history,
             py::arg("module_name"), py::arg("last_n") = 100)
        .def("compute_cross_prediction", &omnimind::SharedWorkspaceCore::compute_cross_prediction,
             py::arg("source_module"), py::arg("target_module"), py::arg("history_window") = 50)
        .def("compute_cross_prediction_causal", &omnimind::SharedWorkspaceCore::compute_cross_prediction_causal,
             py::arg("source_module"), py::arg("target_module"), py::arg("history_window") = 50);
}
