#pragma once
#include "module_state.hpp"
#include <optional>
#include <unordered_map>
#include <vector>

namespace omnimind {

class SharedWorkspaceCore {
public:
    explicit SharedWorkspaceCore(const WorkspaceConfig& config);

    void write_module_state(
        const std::string& module_name,
        const std::vector<float>& embedding,
        const MetadataMap& metadata = {}
    );

    std::vector<float> read_module_state(const std::string& module_name) const;
    MetadataMap read_module_metadata(const std::string& module_name) const;
    std::vector<std::string> get_all_modules() const;
    std::vector<ModuleState> get_module_history(const std::string& module_name, std::size_t last_n = 100) const;

    CrossPredictionMetrics compute_cross_prediction(
        const std::string& source_module,
        const std::string& target_module,
        std::size_t history_window = 50
    ) const;

    CrossPredictionMetrics compute_cross_prediction_causal(
        const std::string& source_module,
        const std::string& target_module,
        std::size_t history_window = 50
    ) const;

private:
    std::vector<float> normalize_embedding_dimension(const std::vector<float>& embedding) const;
    CrossPredictionMetrics make_zero_metrics(const std::string& source, const std::string& target) const;

    WorkspaceConfig config_;
    std::unordered_map<std::string, std::vector<float>> embeddings_;
    std::unordered_map<std::string, MetadataMap> metadata_;
    std::vector<ModuleState> history_;
    std::int64_t cycle_count_ {0};
};

} // namespace omnimind
