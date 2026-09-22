//! SinthomeState — Lógica de adaptação do Sinthome (Protocolo 5, Level 3b)
//!
//! O Sinthome é o quarto anel do nó borromeano (R, S, I).
//! Em Level 3a: regra de corte fixa (cut exactly betti_1 edges).
//! Em Level 3b: regra de corte evolui com histórico de falhas.
//!
//! Este módulo roda no userspace (tem std, Vec, String, serde_json).
//! O eBPF (#![no_std]) apenas captura métricas brutas em SomaticMetrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Tipos de dados
// ============================================================================

/// Edge identifier for Borromean topology graph
pub type EdgeId = u64;

/// Memory pressure stall information (0.0 - 1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPressure {
    pub some: f64,
    pub full: f64,
    pub avg_over_10_cycles: f64,
}

/// Thermal wear with hysteresis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalWear {
    pub temperature_celsius: f64,
    pub cumulative_hysteresis: f64,
    pub thermal_throttling_active: bool,
}

/// Evento de patching do Sinthome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinthomePatchEvent {
    pub timestamp: u64,
    pub cycle: u64,
    pub betti_1: u32,
    pub edges_cut: Vec<EdgeId>,
    pub knot_broke_again: bool,
    /// Quantos ciclos até quebrar de novo (0 se não quebrou)
    pub cycles_until_break: u32,
}

/// Trigger para adaptação da regra do Sinthome
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SinthomeAdaptationTrigger {
    /// knot broke > 3 times in last 10 patches
    RecentFailures,
    /// PSI > 0.75 for N cycles
    HighMemoryPressure,
    /// cumulative_hysteresis > 1.0
    ThermalStress,
    /// OOM/crash within 50 cycles after patch
    PostPatchFailure,
    /// explicit OOM kill event
    OOMKillDetected,
}

/// Versão da regra de corte do Sinthome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinthomeRuleVersion {
    pub version: u32,
    pub adapted_at: u64,
    pub cycle: u64,
    pub trigger: SinthomeAdaptationTrigger,
    pub old_cutting_strategy: String,
    pub new_cutting_strategy: String,
    pub metadata: Option<String>,
}

/// Estratégia de corte (evoluível em Level 3b)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CuttingStrategy {
    /// Level 3a: cut exactly betti_1 edges, no overcut
    ExactBetti1,
    /// Level 3b: cut betti_1 + margin (preventivo)
    Betti1WithMargin { margin: u32 },
    /// Level 3b: cut based on edge weights (mais pesado primeiro)
    Weighted { min_weight_threshold: f64 },
    /// Level 3b: cut based on temporal clustering
    TemporalClustering { clustering_threshold: f64 },
    /// Level 3b: estratégia customizada
    Custom { name: String },
}

impl CuttingStrategy {
    /// Quantas edges cortar dado betti_1
    pub fn edges_to_cut(&self, betti_1: u32) -> u32 {
        match self {
            CuttingStrategy::ExactBetti1 => betti_1,
            CuttingStrategy::Betti1WithMargin { margin } => betti_1 + margin,
            CuttingStrategy::Weighted { .. } => betti_1,
            CuttingStrategy::TemporalClustering { .. } => betti_1,
            CuttingStrategy::Custom { .. } => betti_1,
        }
    }

    /// Nome legível para serialização
    pub fn name(&self) -> String {
        match self {
            CuttingStrategy::ExactBetti1 => "exact_betti1".to_string(),
            CuttingStrategy::Betti1WithMargin { margin } => {
                format!("betti1_with_margin_{}", margin)
            }
            CuttingStrategy::Weighted { min_weight_threshold } => {
                format!("weighted_{}", min_weight_threshold)
            }
            CuttingStrategy::TemporalClustering { clustering_threshold } => {
                format!("temporal_clustering_{}", clustering_threshold)
            }
            CuttingStrategy::Custom { name } => format!("custom_{}", name),
        }
    }
}

/// Evento enviado do userspace Rust para Python
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserspaceEvent {
    pub event_type: String,
    pub timestamp: u64,
    pub cycle: u64,
    pub payload: String,
}

impl UserspaceEvent {
    pub fn sinthome_adaptation(
        cycle: u64,
        trigger: SinthomeAdaptationTrigger,
        old_strategy: &CuttingStrategy,
        new_strategy: &CuttingStrategy,
        version: u32,
    ) -> Self {
        let payload = serde_json::json!({
            "trigger": format!("{:?}", trigger),
            "old_strategy": old_strategy.name(),
            "new_strategy": new_strategy.name(),
            "version": version,
        });
        UserspaceEvent {
            event_type: "sinthome_adaptation".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cycle,
            payload: serde_json::to_string(&payload).unwrap_or_default(),
        }
    }

    pub fn new_version_candidate(cycle: u64, pattern_name: &str, confidence: f64) -> Self {
        let payload = serde_json::json!({
            "pattern_name": pattern_name,
            "confidence": confidence,
            "suggested_version": "D30",
        });
        UserspaceEvent {
            event_type: "new_version_candidate".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cycle,
            payload: serde_json::to_string(&payload).unwrap_or_default(),
        }
    }
}

// ============================================================================
// Funções utilitárias
// ============================================================================

/// Verifica se deve adaptar regra do Sinthome baseado em falhas recentes
pub fn should_adapt_sinthome(recent_failures: usize, threshold: usize) -> bool {
    recent_failures > threshold
}

/// Calcula estratégia de corte baseada em memória e térmica
pub fn adapt_cutting_strategy(
    base_strategy: &CuttingStrategy,
    memory_pressure: &MemoryPressure,
    thermal_wear: &ThermalWear,
) -> CuttingStrategy {
    if memory_pressure.avg_over_10_cycles > 0.75 {
        return CuttingStrategy::Betti1WithMargin { margin: 2 };
    }
    if thermal_wear.cumulative_hysteresis > 1.0 {
        return CuttingStrategy::ExactBetti1;
    }
    base_strategy.clone()
}

// ============================================================================
// Métricas de performance do Sinthome
// ============================================================================

/// Métricas para medir se a adaptação realmente melhora a resiliência.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinthomeMetrics {
    pub total_patches: u64,
    pub total_adaptations: u64,
    pub avg_cycles_between_adaptations: f64,
    /// Taxa de knot_broke_again antes da última adaptação
    pub knot_broke_rate_before: f64,
    /// Taxa de knot_broke_again depois da última adaptação
    pub knot_broke_rate_after: f64,
    /// (edges_cut - betti_1) / betti_1 * 100
    pub overcut_percentage: f64,
}

impl SinthomeMetrics {
    pub fn empty() -> Self {
        SinthomeMetrics {
            total_patches: 0,
            total_adaptations: 0,
            avg_cycles_between_adaptations: 0.0,
            knot_broke_rate_before: 0.0,
            knot_broke_rate_after: 0.0,
            overcut_percentage: 0.0,
        }
    }
}

// ============================================================================
// Registro de adaptação (para rollback e métricas)
// ============================================================================

/// Registro de uma adaptação realizada (para rollback e auditoria).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRecord {
    pub version: u32,
    pub cycle: u64,
    pub trigger: SinthomeAdaptationTrigger,
    pub old_strategy: CuttingStrategy,
    pub new_strategy: CuttingStrategy,
    /// Taxa de falhas nos 100 ciclos antes da adaptação
    pub broke_rate_before: f64,
}

// ============================================================================
// Estado global do Sinthome
// ============================================================================

/// Estado global do Sinthome (compartilhado entre threads)
pub struct SinthomeState {
    pub current_version: u32,
    pub current_strategy: CuttingStrategy,
    pub patch_history: Vec<SinthomePatchEvent>,
    pub last_adaptation_cycle: u64,
    /// Histórico de adaptações (para rollback e métricas)
    pub adaptations: Vec<AdaptationRecord>,
    /// Threshold base para recent_failures (default 3)
    pub base_failure_threshold: usize,
}

impl SinthomeState {
    pub fn new() -> Self {
        SinthomeState {
            current_version: 1,
            current_strategy: CuttingStrategy::ExactBetti1,
            patch_history: Vec::new(),
            last_adaptation_cycle: 0,
            adaptations: Vec::new(),
            base_failure_threshold: 3,
        }
    }

    /// Adiciona evento de patching e verifica se deve adaptar
    pub fn add_patch_event(
        &mut self,
        event: SinthomePatchEvent,
        current_cycle: u64,
    ) -> Option<UserspaceEvent> {
        let should_adapt = self.check_adaptation_trigger(&event, current_cycle);
        self.patch_history.push(event);

        // Manter apenas últimas 100 iterações para memória constante
        if self.patch_history.len() > 100 {
            self.patch_history.remove(0);
        }

        if should_adapt {
            let event = self.adapt_rule(current_cycle);
            // Verificar rollback após cycles_since_adaptation >= 100
            if let Some(rollback_event) = self.maybe_rollback(current_cycle) {
                // Rollback tem prioridade — retorna o rollback ao invés da adaptação original
                return Some(rollback_event);
            }
            Some(event)
        } else {
            // Mesmo sem adaptação, verificar rollback (meta-aprendizado)
            if let Some(rollback_event) = self.maybe_rollback(current_cycle) {
                return Some(rollback_event);
            }
            None
        }
    }

    /// Threshold dinâmico: ajusta sensibilidade baseado em adaptações recentes.
    /// Se poucas adaptações recentes (<2): mais sensível (threshold - 1)
    /// Se muitas adaptações recentes (>5): menos sensível (threshold + 2)
    /// Caso contrário: mantém threshold base
    fn calculate_failure_threshold(&self, base_threshold: usize) -> usize {
        let recent_adaptations = self.adaptations_last_100_cycles();

        if recent_adaptations < 2 {
            // Sistema estável → mais sensível
            base_threshold.saturating_sub(1)
        } else if recent_adaptations > 5 {
            // Muitas adaptações → menos sensível (evitar oscilação)
            base_threshold + 2
        } else {
            base_threshold
        }
    }

    /// Conta adaptações nos últimos 100 ciclos (baseado em patch_history)
    fn adaptations_last_100_cycles(&self) -> usize {
        if self.patch_history.is_empty() {
            return 0;
        }
        let latest_cycle = self.patch_history.last().map(|p| p.cycle).unwrap_or(0);
        let threshold_cycle = latest_cycle.saturating_sub(100);
        self.adaptations
            .iter()
            .filter(|a| a.cycle >= threshold_cycle)
            .count()
    }

    /// Verifica triggers de adaptação
    fn check_adaptation_trigger(&self, event: &SinthomePatchEvent, _current_cycle: u64) -> bool {
        // Trigger 1: Recent failures com threshold dinâmico
        let recent_failures = self
            .patch_history
            .iter()
            .rev()
            .take(10)
            .filter(|p| p.knot_broke_again)
            .count();
        let threshold = self.calculate_failure_threshold(self.base_failure_threshold);
        if should_adapt_sinthome(recent_failures, threshold) {
            return true;
        }

        // Trigger 2: Post-patch failure (quebrou em < 50 ciclos após patch)
        if event.knot_broke_again && event.cycles_until_break < 50 {
            return true;
        }

        false
    }

    /// Adapta regra do Sinthome
    fn adapt_rule(&mut self, current_cycle: u64) -> UserspaceEvent {
        let old_strategy = self.current_strategy.clone();

        // Analisar últimas 10 falhas para decidir nova estratégia
        let recent_failures: Vec<_> = self
            .patch_history
            .iter()
            .rev()
            .take(10)
            .filter(|p| p.knot_broke_again)
            .collect();

        let new_strategy = if recent_failures.len() > 5 {
            // Muitas falhas → overcut preventivo
            CuttingStrategy::Betti1WithMargin { margin: 3 }
        } else if recent_failures.len() > 3 {
            // Falhas moderadas → overcut leve
            CuttingStrategy::Betti1WithMargin { margin: 1 }
        } else {
            // Poucas falhas → mantém exato
            CuttingStrategy::ExactBetti1
        };

        let trigger = if recent_failures.len() > 5 {
            SinthomeAdaptationTrigger::RecentFailures
        } else {
            SinthomeAdaptationTrigger::PostPatchFailure
        };

        // Calcular broke_rate_before (taxa nos 100 ciclos antes da adaptação)
        let broke_rate_before = self.knot_broke_rate_in_range(
            current_cycle.saturating_sub(100),
            current_cycle,
        );

        // Registrar adaptação para rollback e métricas
        self.adaptations.push(AdaptationRecord {
            version: self.current_version + 1,
            cycle: current_cycle,
            trigger: trigger.clone(),
            old_strategy: old_strategy.clone(),
            new_strategy: new_strategy.clone(),
            broke_rate_before,
        });

        // Manter apenas últimas 50 adaptações
        if self.adaptations.len() > 50 {
            self.adaptations.remove(0);
        }

        self.current_version += 1;
        self.current_strategy = new_strategy.clone();
        self.last_adaptation_cycle = current_cycle;

        UserspaceEvent::sinthome_adaptation(
            current_cycle,
            trigger,
            &old_strategy,
            &new_strategy,
            self.current_version,
        )
    }

    /// Rollback de estratégia: se a última adaptação piorou performance,
    /// volta para a estratégia anterior (meta-aprendizado).
    /// Só avalia após 100 ciclos desde a adaptação.
    fn maybe_rollback(&mut self, current_cycle: u64) -> Option<UserspaceEvent> {
        let last_adaptation = self.adaptations.last()?;
        let cycles_since_adaptation = current_cycle.saturating_sub(last_adaptation.cycle);

        // Muito recente para avaliar
        if cycles_since_adaptation < 100 {
            return None;
        }

        let broke_rate_before = last_adaptation.broke_rate_before;
        let broke_rate_after =
            self.knot_broke_rate_in_range(last_adaptation.cycle, current_cycle);

        // Se piorou > 20% → rollback
        if broke_rate_after > broke_rate_before * 1.2 && broke_rate_before > 0.0 {
            let old_strategy = self.current_strategy.clone();
            let previous_strategy = last_adaptation.old_strategy.clone();

            self.current_version += 1;
            self.current_strategy = previous_strategy.clone();
            self.last_adaptation_cycle = current_cycle;

            // Registrar rollback como nova adaptação
            self.adaptations.push(AdaptationRecord {
                version: self.current_version,
                cycle: current_cycle,
                trigger: SinthomeAdaptationTrigger::RecentFailures,
                old_strategy: old_strategy.clone(),
                new_strategy: previous_strategy.clone(),
                broke_rate_before: broke_rate_after,
            });

            return Some(UserspaceEvent::sinthome_adaptation(
                current_cycle,
                SinthomeAdaptationTrigger::RecentFailures,
                &old_strategy,
                &previous_strategy,
                self.current_version,
            ));
        }

        None
    }

    /// Calcula taxa de knot_broke_again em um intervalo de ciclos
    fn knot_broke_rate_in_range(&self, start_cycle: u64, end_cycle: u64) -> f64 {
        let in_range: Vec<_> = self
            .patch_history
            .iter()
            .filter(|p| p.cycle >= start_cycle && p.cycle < end_cycle)
            .collect();
        if in_range.is_empty() {
            return 0.0;
        }
        let broke = in_range.iter().filter(|p| p.knot_broke_again).count();
        broke as f64 / in_range.len() as f64
    }

    /// Calcula métricas de performance atuais
    pub fn get_metrics(&self) -> SinthomeMetrics {
        let total_patches = self.patch_history.len() as u64;
        let total_adaptations = self.adaptations.len() as u64;

        let avg_cycles_between = if self.adaptations.len() > 1 {
            let mut sum = 0u64;
            for i in 1..self.adaptations.len() {
                sum += self.adaptations[i]
                    .cycle
                    .saturating_sub(self.adaptations[i - 1].cycle);
            }
            sum as f64 / (self.adaptations.len() - 1) as f64
        } else {
            0.0
        };

        let (rate_before, rate_after) = if let Some(last) = self.adaptations.last() {
            let after = self.knot_broke_rate_in_range(last.cycle, last.cycle + 100);
            (last.broke_rate_before, after)
        } else {
            (0.0, 0.0)
        };

        let overcut = match &self.current_strategy {
            CuttingStrategy::ExactBetti1 => 0.0,
            CuttingStrategy::Betti1WithMargin { margin } => *margin as f64 * 100.0,
            _ => 0.0,
        };

        SinthomeMetrics {
            total_patches,
            total_adaptations,
            avg_cycles_between_adaptations: avg_cycles_between,
            knot_broke_rate_before: rate_before,
            knot_broke_rate_after: rate_after,
            overcut_percentage: overcut,
        }
    }

    /// Retorna estado atual para auditoria
    pub fn get_state(&self) -> serde_json::Value {
        let recent_failures = self
            .patch_history
            .iter()
            .rev()
            .take(10)
            .filter(|p| p.knot_broke_again)
            .count();
        let metrics = self.get_metrics();
        let dynamic_threshold =
            self.calculate_failure_threshold(self.base_failure_threshold);

        serde_json::json!({
            "current_version": self.current_version,
            "current_strategy": self.current_strategy.name(),
            "patch_history_length": self.patch_history.len(),
            "last_adaptation_cycle": self.last_adaptation_cycle,
            "recent_failures": recent_failures,
            "dynamic_threshold": dynamic_threshold,
            "base_failure_threshold": self.base_failure_threshold,
            "adaptations_count": self.adaptations.len(),
            "metrics": {
                "total_patches": metrics.total_patches,
                "total_adaptations": metrics.total_adaptations,
                "avg_cycles_between_adaptations": metrics.avg_cycles_between_adaptations,
                "knot_broke_rate_before": metrics.knot_broke_rate_before,
                "knot_broke_rate_after": metrics.knot_broke_rate_after,
                "overcut_percentage": metrics.overcut_percentage,
            },
        })
    }
}

/// Cria um SinthomePatchEvent a partir de dados do grafo borromeano
pub fn create_patch_event(
    cycle: u64,
    betti_1: u32,
    edges_cut: Vec<u64>,
    knot_broke_again: bool,
    cycles_until_break: u32,
) -> SinthomePatchEvent {
    SinthomePatchEvent {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        cycle,
        betti_1,
        edges_cut,
        knot_broke_again,
        cycles_until_break,
    }
}

// ============================================================================
// PatternDetector (Protocolo 4: Meta-Grupo)
// ============================================================================

/// Detecta novos padrões de syscall que podem indicar emergência de versão
pub struct PatternDetector {
    known_patterns: Vec<String>,
    syscall_counts: HashMap<String, u64>,
}

impl PatternDetector {
    pub fn new() -> Self {
        PatternDetector {
            known_patterns: vec![
                "memory".to_string(),
                "network".to_string(),
                "io".to_string(),
                "ipc".to_string(),
            ],
            syscall_counts: HashMap::new(),
        }
    }

    /// Analisa syscall e verifica se é novo padrão
    pub fn analyze_syscall(&mut self, syscall_name: &str) -> Option<UserspaceEvent> {
        *self
            .syscall_counts
            .entry(syscall_name.to_string())
            .or_insert(0) += 1;

        // Verificar se syscall forma novo padrão (ex: quantum_ipc)
        if syscall_name.starts_with("quantum_")
            && !self.known_patterns.contains(&"quantum_ipc".to_string())
        {
            self.known_patterns.push("quantum_ipc".to_string());

            // Sinalizar emergência de nova versão
            return Some(UserspaceEvent::new_version_candidate(
                0,
                "quantum_ipc",
                0.85,
            ));
        }

        None
    }
}

// ============================================================================
// Testes
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_adapt_sinthome() {
        assert_eq!(should_adapt_sinthome(3, 3), false);
        assert_eq!(should_adapt_sinthome(4, 3), true);
        assert_eq!(should_adapt_sinthome(5, 3), true);
    }

    #[test]
    fn test_cutting_strategy_edges_to_cut() {
        let exact = CuttingStrategy::ExactBetti1;
        assert_eq!(exact.edges_to_cut(5), 5);

        let margin = CuttingStrategy::Betti1WithMargin { margin: 2 };
        assert_eq!(margin.edges_to_cut(5), 7);
    }

    #[test]
    fn test_adapt_cutting_strategy() {
        let base = CuttingStrategy::ExactBetti1;
        let mem_high = MemoryPressure {
            some: 0.8,
            full: 0.9,
            avg_over_10_cycles: 0.85,
        };
        let mem_low = MemoryPressure {
            some: 0.3,
            full: 0.4,
            avg_over_10_cycles: 0.35,
        };
        let thermal_ok = ThermalWear {
            temperature_celsius: 65.0,
            cumulative_hysteresis: 0.5,
            thermal_throttling_active: false,
        };

        // Memória alta → overcut
        let adapted = adapt_cutting_strategy(&base, &mem_high, &thermal_ok);
        assert!(matches!(adapted, CuttingStrategy::Betti1WithMargin { .. }));

        // Memória baixa → mantém base
        let adapted = adapt_cutting_strategy(&base, &mem_low, &thermal_ok);
        assert!(matches!(adapted, CuttingStrategy::ExactBetti1));
    }

    #[test]
    fn test_sinthome_state_new() {
        let state = SinthomeState::new();
        assert_eq!(state.current_version, 1);
        assert_eq!(state.current_strategy, CuttingStrategy::ExactBetti1);
        assert!(state.patch_history.is_empty());
    }

    #[test]
    fn test_add_patch_event_no_adapt() {
        let mut state = SinthomeState::new();
        let event = create_patch_event(1, 2, vec![0, 1], false, 0);
        let result = state.add_patch_event(event, 1);
        assert!(result.is_none());
        assert_eq!(state.current_version, 1);
    }

    #[test]
    fn test_add_patch_event_adapt_on_failures() {
        let mut state = SinthomeState::new();

        // Adicionar 4 falhas nas últimas 10 iterações
        for i in 0..4 {
            let event = create_patch_event(i, 1, vec![0], true, 30);
            state.add_patch_event(event, i);
        }

        // A 4ª falha deveria ter disparado adaptação (recent_failures > 3)
        assert!(state.current_version > 1);
    }

    #[test]
    fn test_pattern_detector_known() {
        let mut detector = PatternDetector::new();
        assert!(detector.analyze_syscall("read").is_none());
        assert!(detector.analyze_syscall("write").is_none());
    }

    #[test]
    fn test_pattern_detector_new_pattern() {
        let mut detector = PatternDetector::new();
        let event = detector.analyze_syscall("quantum_entangle");
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.event_type, "new_version_candidate");
    }

    #[test]
    fn test_dynamic_threshold_stable() {
        // Sistema estável (0 adaptações recentes) → threshold - 1 (mais sensível)
        let state = SinthomeState::new();
        let threshold = state.calculate_failure_threshold(3);
        assert_eq!(threshold, 2); // 3 - 1 = 2
    }

    #[test]
    fn test_dynamic_threshold_oscillating() {
        // Muitas adaptações recentes (>5) → threshold + 2 (menos sensível)
        let mut state = SinthomeState::new();
        // Simular 6 adaptações nos últimos 100 ciclos
        for i in 0..6u64 {
            state.adaptations.push(AdaptationRecord {
                version: (i + 2) as u32,
                cycle: i * 10,
                trigger: SinthomeAdaptationTrigger::RecentFailures,
                old_strategy: CuttingStrategy::ExactBetti1,
                new_strategy: CuttingStrategy::Betti1WithMargin { margin: 1 },
                broke_rate_before: 0.3,
            });
        }
        // Adicionar patch event para ter latest_cycle
        state.patch_history.push(create_patch_event(50, 1, vec![0], false, 0));

        let threshold = state.calculate_failure_threshold(3);
        assert_eq!(threshold, 5); // 3 + 2 = 5
    }

    #[test]
    fn test_knot_broke_rate_in_range() {
        let mut state = SinthomeState::new();
        // 10 patches, 3 com knot_broke_again
        for i in 0..10 {
            let broke = i % 3 == 0;
            state.patch_history.push(create_patch_event(i, 1, vec![0], broke, 30));
        }
        let rate = state.knot_broke_rate_in_range(0, 10);
        assert!((rate - 0.4).abs() < 0.01); // 4 broke / 10 total = 0.4
    }

    #[test]
    fn test_get_metrics_empty() {
        let state = SinthomeState::new();
        let metrics = state.get_metrics();
        assert_eq!(metrics.total_patches, 0);
        assert_eq!(metrics.total_adaptations, 0);
        assert_eq!(metrics.avg_cycles_between_adaptations, 0.0);
    }

    #[test]
    fn test_get_metrics_with_adaptations() {
        let mut state = SinthomeState::new();
        // Simular 20 patches
        for i in 0..20 {
            let broke = i % 4 == 0; // 25% broke rate
            state.patch_history.push(create_patch_event(i, 1, vec![0], broke, 30));
        }
        // Simular 2 adaptações
        state.adaptations.push(AdaptationRecord {
            version: 2,
            cycle: 5,
            trigger: SinthomeAdaptationTrigger::RecentFailures,
            old_strategy: CuttingStrategy::ExactBetti1,
            new_strategy: CuttingStrategy::Betti1WithMargin { margin: 1 },
            broke_rate_before: 0.25,
        });
        state.adaptations.push(AdaptationRecord {
            version: 3,
            cycle: 15,
            trigger: SinthomeAdaptationTrigger::RecentFailures,
            old_strategy: CuttingStrategy::Betti1WithMargin { margin: 1 },
            new_strategy: CuttingStrategy::Betti1WithMargin { margin: 3 },
            broke_rate_before: 0.25,
        });
        state.current_strategy = CuttingStrategy::Betti1WithMargin { margin: 3 };

        let metrics = state.get_metrics();
        assert_eq!(metrics.total_patches, 20);
        assert_eq!(metrics.total_adaptations, 2);
        assert!((metrics.avg_cycles_between_adaptations - 10.0).abs() < 0.01); // (15-5)/1 = 10
        assert_eq!(metrics.overcut_percentage, 300.0); // margin 3 * 100
    }

    #[test]
    fn test_rollback_not_triggered_too_recent() {
        let mut state = SinthomeState::new();
        state.adaptations.push(AdaptationRecord {
            version: 2,
            cycle: 50,
            trigger: SinthomeAdaptationTrigger::RecentFailures,
            old_strategy: CuttingStrategy::ExactBetti1,
            new_strategy: CuttingStrategy::Betti1WithMargin { margin: 1 },
            broke_rate_before: 0.2,
        });
        // Apenas 50 ciclos desde adaptação (< 100) → não deve rollback
        let result = state.maybe_rollback(100);
        assert!(result.is_none());
    }

    #[test]
    fn test_rollback_triggered_when_worse() {
        let mut state = SinthomeState::new();
        state.adaptations.push(AdaptationRecord {
            version: 2,
            cycle: 0,
            trigger: SinthomeAdaptationTrigger::RecentFailures,
            old_strategy: CuttingStrategy::ExactBetti1,
            new_strategy: CuttingStrategy::Betti1WithMargin { margin: 1 },
            broke_rate_before: 0.2,
        });
        state.current_strategy = CuttingStrategy::Betti1WithMargin { margin: 1 };
        // 150 patches: 50 antes (20% broke), 100 depois (40% broke) → piorou > 20%
        for i in 0..50 {
            let broke = i % 5 == 0; // 20% broke rate
            state.patch_history.push(create_patch_event(i, 1, vec![0], broke, 30));
        }
        for i in 50..150 {
            let broke = i % 3 == 0; // ~33% broke rate (piorou > 20%)
            state.patch_history.push(create_patch_event(i, 1, vec![0], broke, 30));
        }

        // 150 ciclos desde adaptação (>= 100) → pode avaliar rollback
        let result = state.maybe_rollback(150);
        assert!(result.is_some(), "rollback should trigger when rate worsens");
        assert_eq!(state.current_strategy, CuttingStrategy::ExactBetti1);
    }

    #[test]
    fn test_rollback_not_triggered_when_better() {
        let mut state = SinthomeState::new();
        state.adaptations.push(AdaptationRecord {
            version: 2,
            cycle: 0,
            trigger: SinthomeAdaptationTrigger::RecentFailures,
            old_strategy: CuttingStrategy::ExactBetti1,
            new_strategy: CuttingStrategy::Betti1WithMargin { margin: 1 },
            broke_rate_before: 0.4,
        });
        state.current_strategy = CuttingStrategy::Betti1WithMargin { margin: 1 };
        // 50 patches antes (40% broke), 100 depois (20% broke) → melhorou
        for i in 0..50 {
            let broke = i % 3 == 0; // ~33% broke rate
            state.patch_history.push(create_patch_event(i, 1, vec![0], broke, 30));
        }
        for i in 50..150 {
            let broke = i % 5 == 0; // 20% broke rate (melhorou)
            state.patch_history.push(create_patch_event(i, 1, vec![0], broke, 30));
        }

        let result = state.maybe_rollback(150);
        assert!(result.is_none(), "rollback should not trigger when rate improves");
    }

    #[test]
    fn test_get_state_includes_metrics_and_threshold() {
        let mut state = SinthomeState::new();
        state.patch_history.push(create_patch_event(1, 2, vec![0, 1], false, 0));
        let state_json = state.get_state();
        assert!(state_json.get("metrics").is_some());
        assert!(state_json.get("dynamic_threshold").is_some());
        assert!(state_json.get("base_failure_threshold").is_some());
        assert!(state_json.get("adaptations_count").is_some());
    }
}
