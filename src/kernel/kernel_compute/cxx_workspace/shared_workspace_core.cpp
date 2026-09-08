#include "shared_workspace_core.hpp"
#include <chrono>
#include <cmath>
#include <numeric>
#include <algorithm>

namespace omnimind {

SharedWorkspaceCore::SharedWorkspaceCore(const WorkspaceConfig& config)
    : config_(config) {}

std::vector<float> SharedWorkspaceCore::normalize_embedding_dimension(const std::vector<float>& embedding) const {
    std::vector<float> normalized = embedding;
    if (normalized.size() > config_.embedding_dim) {
        normalized.resize(config_.embedding_dim);
    } else if (normalized.size() < config_.embedding_dim) {
        normalized.resize(config_.embedding_dim, 0.0f);
    }
    return normalized;
}

void SharedWorkspaceCore::write_module_state(
    const std::string& module_name,
    const std::vector<float>& embedding,
    const MetadataMap& metadata
) {
    auto normalized = normalize_embedding_dimension(embedding);
    embeddings_[module_name] = normalized;
    metadata_[module_name] = metadata;

    double now = std::chrono::duration<double>(
        std::chrono::system_clock::now().time_since_epoch()
    ).count();

    ModuleState state{
        module_name,
        normalized,
        now,
        cycle_count_,
        metadata
    };

    if (history_.size() >= config_.max_history_size) {
        // Simple circular behavior or shift (optimizations can use a circular buffer later)
        history_.erase(history_.begin());
    }
    history_.push_back(state);
    cycle_count_++;
}

std::vector<float> SharedWorkspaceCore::read_module_state(const std::string& module_name) const {
    auto it = embeddings_.find(module_name);
    if (it != embeddings_.end()) {
        return it->second;
    }
    return std::vector<float>(config_.embedding_dim, 0.0f);
}

MetadataMap SharedWorkspaceCore::read_module_metadata(const std::string& module_name) const {
    auto it = metadata_.find(module_name);
    if (it != metadata_.end()) {
        return it->second;
    }
    return {};
}

std::vector<std::string> SharedWorkspaceCore::get_all_modules() const {
    std::vector<std::string> modules;
    modules.reserve(embeddings_.size());
    for (const auto& pair : embeddings_) {
        modules.push_back(pair.first);
    }
    return modules;
}

std::vector<ModuleState> SharedWorkspaceCore::get_module_history(const std::string& module_name, std::size_t last_n) const {
    std::vector<ModuleState> result;
    for (auto it = history_.rbegin(); it != history_.rend(); ++it) {
        if (it->module_name == module_name) {
            result.push_back(*it);
            if (result.size() >= last_n) {
                break;
            }
        }
    }
    std::reverse(result.begin(), result.end());
    return result;
}

CrossPredictionMetrics SharedWorkspaceCore::make_zero_metrics(const std::string& source, const std::string& target) const {
    CrossPredictionMetrics m;
    m.source_module = source;
    m.target_module = target;
    m.timestamp = std::chrono::duration<double>(
        std::chrono::system_clock::now().time_since_epoch()
    ).count();
    return m;
}

CrossPredictionMetrics SharedWorkspaceCore::compute_cross_prediction(
    const std::string& source_module,
    const std::string& target_module,
    std::size_t history_window
) const {
    auto source_hist = get_module_history(source_module, history_window + 1);
    auto target_hist = get_module_history(target_module, history_window + 1);

    std::size_t window = std::min(source_hist.size(), target_hist.size());
    if (window < 2) {
        return make_zero_metrics(source_module, target_module);
    }
    // Alignment: source[:-1] and target[1:]
    window = window - 1;

    // Calculate correlation element-wise across the embedding dimensions
    std::size_t dim = config_.embedding_dim;
    std::vector<double> correlations;
    correlations.reserve(dim);

    for (std::size_t d = 0; d < dim; ++d) {
        double sum_x = 0, sum_y = 0, sum_x2 = 0, sum_y2 = 0, sum_xy = 0;
        for (std::size_t t = 0; t < window; ++t) {
            double x = source_hist[t].embedding[d];
            double y = target_hist[t + 1].embedding[d];
            sum_x += x;
            sum_y += y;
            sum_x2 += x * x;
            sum_y2 += y * y;
            sum_xy += x * y;
        }

        double mean_x = sum_x / window;
        double mean_y = sum_y / window;
        
        // standard deviation threshold check (1e-8)
        double variance_x = (sum_x2 / window) - (mean_x * mean_x);
        double variance_y = (sum_y2 / window) - (mean_y * mean_y);

        if (std::sqrt(std::max(0.0, variance_x)) > 1e-8 && std::sqrt(std::max(0.0, variance_y)) > 1e-8) {
            double cov = (sum_xy / window) - (mean_x * mean_y);
            double std_x = std::sqrt(std::max(0.0, variance_x));
            double std_y = std::sqrt(std::max(0.0, variance_y));
            double corr = cov / (std_x * std_y);
            if (!std::isnan(corr)) {
                correlations.push_back(std::abs(corr));
            }
        }
    }

    double final_correlation = 0.0;
    if (!correlations.empty()) {
        double sum = 0.0;
        for (double c : correlations) sum += c;
        final_correlation = sum / correlations.size();
    }

    CrossPredictionMetrics m = make_zero_metrics(source_module, target_module);
    m.correlation = final_correlation;
    m.mutual_information = (window >= 4) ? (final_correlation * 0.8) : 0.0; // matching python >=5 points (window >=4 since window is history-1)
    
    // R-squared matrix regression is omitted here until LibTorch integration.
    m.r_squared = 0.0; 

    return m;
}

CrossPredictionMetrics SharedWorkspaceCore::compute_cross_prediction_causal(
    const std::string& source_module,
    const std::string& target_module,
    std::size_t history_window
) const {
    return make_zero_metrics(source_module, target_module);
}

} // namespace omnimind
