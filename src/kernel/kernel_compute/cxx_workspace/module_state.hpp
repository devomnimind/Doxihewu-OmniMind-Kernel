#pragma once
#include <cstdint>
#include <string>
#include <unordered_map>
#include <vector>

namespace omnimind {

using MetadataMap = std::unordered_map<std::string, std::string>;

struct ModuleState {
    std::string module_name;
    std::vector<float> embedding;
    double timestamp {0.0};
    std::int64_t cycle {0};
    MetadataMap metadata;
};

struct CrossPredictionMetrics {
    std::string source_module;
    std::string target_module;
    double r_squared {0.0};
    double correlation {0.0};
    double mutual_information {0.0};
    double granger_causality {0.0};
    double transfer_entropy {0.0};
    double score {0.0};
    double timestamp {0.0};
};

struct WorkspaceConfig {
    std::size_t embedding_dim {256};
    std::size_t max_history_size {14700};
};

} // namespace omnimind
