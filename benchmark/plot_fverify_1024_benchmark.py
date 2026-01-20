#!/usr/bin/env python3
"""
Visualization of Falcon-1024 Fast Verify Benchmark Results

This script generates a publication-quality plot comparing:
1. fverify (fast verify) with varying numbers of indices
2. Standard verify (as a baseline reference)
3. C FFI verify (as an alternative implementation reference)
"""

import json
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# Configure matplotlib for publication-quality figures
plt.style.use('seaborn-v0_8-whitegrid')
plt.rcParams.update({
    'font.size': 11,
    'axes.labelsize': 12,
    'axes.titlesize': 14,
    'legend.fontsize': 10,
    'figure.figsize': (10, 6),
    'figure.dpi': 150,
})

def load_criterion_estimate(path: Path) -> dict:
    """Load timing estimates from Criterion's JSON output."""
    estimates_path = path / "new" / "estimates.json"
    if not estimates_path.exists():
        estimates_path = path / "base" / "estimates.json"
    
    with open(estimates_path) as f:
        data = json.load(f)
    
    mean = data["mean"]["point_estimate"]
    ci_lower = data["mean"]["confidence_interval"]["lower_bound"]
    ci_upper = data["mean"]["confidence_interval"]["upper_bound"]
    
    return {
        "mean": mean,
        "ci_lower": ci_lower,
        "ci_upper": ci_upper,
        "error_lower": mean - ci_lower,
        "error_upper": ci_upper - mean,
    }

def main():
    criterion_dir = Path("/Users/vamsi/Github/falcon-rust/target/criterion")
    
    # Load fverify 1024 data for different index counts
    index_counts = [1, 2, 4, 8, 16, 32, 48, 64]
    fverify_data = []
    
    for n in index_counts:
        path = criterion_dir / "falcon-rust" / f"fast verify 1024 - {n} indices"
        if path.exists():
            estimate = load_criterion_estimate(path)
            estimate["indices"] = n
            fverify_data.append(estimate)
            print(f"fverify 1024 ({n} indices): {estimate['mean']/1000:.2f} µs")
    
    # Load baseline verify 1024
    verify_1024_path = criterion_dir / "falcon-rust" / "verify 1024"
    verify_1024 = load_criterion_estimate(verify_1024_path)
    print(f"verify 1024: {verify_1024['mean']/1000:.2f} µs")
    
    # Load C FFI verify 1024
    ffi_verify_path = criterion_dir / "c ffi" / "verify 1024"
    ffi_verify = load_criterion_estimate(ffi_verify_path)
    print(f"C FFI verify 1024: {ffi_verify['mean']/1000:.2f} µs")
    
    # Prepare data for plotting
    indices = np.array([d["indices"] for d in fverify_data])
    means = np.array([d["mean"] for d in fverify_data]) / 1000  # Convert to µs
    errors_lower = np.array([d["error_lower"] for d in fverify_data]) / 1000
    errors_upper = np.array([d["error_upper"] for d in fverify_data]) / 1000
    
    verify_mean = verify_1024["mean"] / 1000
    verify_ci = (verify_1024["ci_upper"] - verify_1024["ci_lower"]) / 2 / 1000
    
    ffi_mean = ffi_verify["mean"] / 1000
    ffi_ci = (ffi_verify["ci_upper"] - ffi_verify["ci_lower"]) / 2 / 1000
    
    # Create the plot
    fig, ax = plt.subplots()
    
    # Plot fverify data with error bars
    ax.errorbar(indices, means, yerr=[errors_lower, errors_upper], 
                fmt='o-', color='#2E86AB', linewidth=2, markersize=8,
                capsize=4, capthick=1.5, label='fverify 1024 (fast verify)',
                zorder=3)
    
    # Plot verify 1024 baseline as horizontal line with uncertainty band
    ax.axhline(y=verify_mean, color='#E94F37', linestyle='--', linewidth=2,
               label=f'verify 1024 (Rust): {verify_mean:.1f} µs', zorder=2)
    ax.fill_between(ax.get_xlim(), verify_mean - verify_ci, verify_mean + verify_ci,
                    color='#E94F37', alpha=0.15, zorder=1)
    
    # Plot C FFI verify baseline
    ax.axhline(y=ffi_mean, color='#8B5CF6', linestyle=':', linewidth=2,
               label=f'verify 1024 (C FFI): {ffi_mean:.1f} µs', zorder=2)
    ax.fill_between(ax.get_xlim(), ffi_mean - ffi_ci, ffi_mean + ffi_ci,
                    color='#8B5CF6', alpha=0.15, zorder=1)
    
    # Find and annotate breakeven point
    for i in range(len(means) - 1):
        if means[i] < verify_mean <= means[i + 1]:
            x1, x2 = indices[i], indices[i + 1]
            y1, y2 = means[i], means[i + 1]
            breakeven = x1 + (verify_mean - y1) * (x2 - x1) / (y2 - y1)
            
            ax.axvline(x=breakeven, color='#44AF69', linestyle='-.',
                       linewidth=1.5, alpha=0.7)
            ax.annotate(f'Breakeven ≈ {breakeven:.0f} indices',
                       xy=(breakeven, verify_mean),
                       xytext=(breakeven + 5, verify_mean * 0.7),
                       fontsize=10, color='#44AF69',
                       arrowprops=dict(arrowstyle='->', color='#44AF69', lw=1.5),
                       bbox=dict(boxstyle='round,pad=0.3', facecolor='white', 
                                edgecolor='#44AF69', alpha=0.9))
            break
    
    # Labels and title
    ax.set_xlabel('Number of Indices Verified', fontweight='bold')
    ax.set_ylabel('Time (µs)', fontweight='bold')
    ax.set_title('Falcon-1024: Fast Verify Performance vs Number of Indices\n'
                 '(Lower is better)', fontweight='bold')
    
    # Use log scale for x-axis
    ax.set_xscale('log', base=2)
    ax.set_xticks(indices)
    ax.set_xticklabels(indices)
    
    # Set y-axis to start from 0
    ax.set_ylim(bottom=0)
    
    # Legend
    ax.legend(loc='upper left', framealpha=0.95)
    
    # Add grid
    ax.grid(True, alpha=0.3, linestyle='-')
    ax.set_axisbelow(True)
    
    # Add explanatory text
    textstr = ('fverify allows verifying a subset of polynomial coefficients.\n'
               'Useful when probabilistic verification is acceptable.\n'
               'Below the breakeven point, fverify is faster than full verify.')
    props = dict(boxstyle='round,pad=0.5', facecolor='wheat', alpha=0.8)
    ax.text(0.98, 0.02, textstr, transform=ax.transAxes, fontsize=9,
            verticalalignment='bottom', horizontalalignment='right', bbox=props)
    
    plt.tight_layout()
    
    # Save the figure
    output_path = Path("/Users/vamsi/Github/falcon-rust/benchmark/fverify_1024_benchmark.png")
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"\nPlot saved to: {output_path}")
    
    pdf_path = output_path.with_suffix('.pdf')
    plt.savefig(pdf_path, bbox_inches='tight')
    print(f"PDF saved to: {pdf_path}")
    
    plt.show()

if __name__ == "__main__":
    main()
