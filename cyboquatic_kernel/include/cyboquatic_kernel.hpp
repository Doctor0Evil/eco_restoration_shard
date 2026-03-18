// File: cyboquatic_kernel/include/cyboquatic_kernel.hpp

#ifndef CYBOQUATIC_KERNEL_HPP
#define CYBOQUATIC_KERNEL_HPP

#include <vector>
#include <string>

struct ReachState {
    double q_m3s;
    double area_m2;
    double c_kg_per_m3;
    double temp_c;
    double t90_days;
};

struct KernelCorridors {
    double q_hard_m3s;
    double c_safe_kg_per_m3;
    double c_hard_kg_per_m3;
    double t90_gold_days;
    double t90_max_days;
};

struct KernelRisks {
    double r_q;
    double r_c;
    double r_t90;
};

struct StepResult {
    ReachState state;
    KernelRisks risks;
    double vt;
};

double normalize_linear(double x, double xsafe, double xgold, double xhard);

KernelRisks compute_risks(const ReachState& s, const KernelCorridors& c);

double compute_vt(const KernelRisks& r, const double* w, std::size_t n);

StepResult advance_reach(
    const ReachState& s0,
    const KernelCorridors& corridors,
    const double* weights,
    std::size_t n_weights,
    double dt_seconds
);

void write_shard_row_csv(
    const std::string& filepath,
    double t_hours,
    const ReachState& s,
    const KernelRisks& r,
    double vt,
    const std::string& hexstamp
);

#endif // CYBOQUATIC_KERNEL_HPP
