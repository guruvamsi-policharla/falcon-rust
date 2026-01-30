#!/usr/bin/env python3
"""
Visualization of Falcon-512 Fast Verify Benchmark Results

This script generates a publication-quality plot comparing:
1. fverify (fast verify) with varying numbers of indices
2. Standard verify (as a baseline reference)
3. C FFI verify (as an alternative implementation reference)

The goal is to help readers understand the performance trade-off:
- fverify is faster when verifying only a subset of indices
- At some "breakeven point", fverify becomes slower than full verify

Scientific justification for visualization choices:
- Error bars show 95% confidence intervals (statistical rigor)
- Horizontal reference lines for verify baselines (easy comparison)
- Shaded regions for uncertainty in baselines
- Log-scale x-axis to show the multiplicative relationship
- Clear annotations for the breakeven point
"""

import json
import os
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
    
    # Extract mean timing and confidence interval (in nanoseconds)
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
    
    # Load fverify 512 data for different index counts
    index_counts = [1, 4, 8, 16, 32, 48, 64]
    fverify_512_data = []
    
    for n in index_counts:
        path = criterion_dir / "falcon-rust" / f"fast verify 512 - {n} indices"
        if path.exists():
            estimate = load_criterion_estimate(path)
            estimate["indices"] = n
            fverify_512_data.append(estimate)
            print(f"fverify 512 ({n} indices): {estimate['mean']/1000:.2f} µs")
    
    # Load fverify 1024 data for different index counts
    fverify_1024_data = []
    
    for n in index_counts:
        path = criterion_dir / "falcon-rust" / f"fast verify 1024 - {n} indices"
        if path.exists():
            estimate = load_criterion_estimate(path)
            estimate["indices"] = n
            fverify_1024_data.append(estimate)
            print(f"fverify 1024 ({n} indices): {estimate['mean']/1000:.2f} µs")
    
    # Load baseline verify 512
    verify_512_path = criterion_dir / "falcon-rust" / "verify 512"
    verify_512 = load_criterion_estimate(verify_512_path)
    print(f"verify 512: {verify_512['mean']/1000:.2f} µs")
    
    # Load baseline verify 1024
    verify_1024_path = criterion_dir / "falcon-rust" / "verify 1024"
    verify_1024 = load_criterion_estimate(verify_1024_path)
    print(f"verify 1024: {verify_1024['mean']/1000:.2f} µs")
    
    # Prepare Falcon-512 data for plotting
    indices_512 = np.array([d["indices"] for d in fverify_512_data])
    means_512 = np.array([d["mean"] for d in fverify_512_data]) / 1000  # Convert to µs
    
    # Prepare Falcon-1024 data for plotting
    indices_1024 = np.array([d["indices"] for d in fverify_1024_data])
    means_1024 = np.array([d["mean"] for d in fverify_1024_data]) / 1000  # Convert to µs
    
    verify_512_mean = verify_512["mean"] / 1000
    verify_1024_mean = verify_1024["mean"] / 1000
    
    # Create the plot
    fig, ax = plt.subplots()
    
    # Plot Falcon-512 fverify data
    ax.plot(indices_512, means_512, 
            'o-', color='#2E86AB', linewidth=2, markersize=8,
            label='Fast Verify Falcon-512',
            zorder=3)
    
    # Plot Falcon-1024 fverify data
    ax.plot(indices_1024, means_1024, 
            's-', color='#F77F00', linewidth=2, markersize=8,
            label='Fast Verify Falcon-1024',
            zorder=3)
    
    # Plot verify 512 baseline as horizontal line
    ax.axhline(y=verify_512_mean, color='#E94F37', linestyle='--', linewidth=2,
               label=f'Verify Falcon-512: {verify_512_mean:.1f} µs', zorder=2)
    
    # Plot verify 1024 baseline as horizontal line
    ax.axhline(y=verify_1024_mean, color='#06A77D', linestyle='--', linewidth=2,
               label=f'Verify Falcon-1024: {verify_1024_mean:.1f} µs', zorder=2)
    
    # Find and annotate breakeven point for Falcon-512 (linear interpolation)
    for i in range(len(means_512) - 1):
        if means_512[i] < verify_512_mean <= means_512[i + 1]:
            # Linear interpolation to find crossover
            x1, x2 = indices_512[i], indices_512[i + 1]
            y1, y2 = means_512[i], means_512[i + 1]
            breakeven_512 = x1 + (verify_512_mean - y1) * (x2 - x1) / (y2 - y1)
            
            # Annotate breakeven point (positioned to the left to stay within plot)
            ax.annotate(f'512 Breakeven ≈ {breakeven_512:.0f}',
                       xy=(breakeven_512, verify_512_mean),
                       xytext=(breakeven_512 * 0.53, verify_512_mean * 1.2),
                       fontsize=9, color='#E94F37',
                       arrowprops=dict(arrowstyle='->', color='#E94F37', lw=1.5),
                       bbox=dict(boxstyle='round,pad=0.3', facecolor='white', 
                                edgecolor='#E94F37', alpha=0.9))
            break
    
    # Find and annotate breakeven point for Falcon-1024 (linear interpolation)
    for i in range(len(means_1024) - 1):
        if means_1024[i] < verify_1024_mean <= means_1024[i + 1]:
            # Linear interpolation to find crossover
            x1, x2 = indices_1024[i], indices_1024[i + 1]
            y1, y2 = means_1024[i], means_1024[i + 1]
            breakeven_1024 = x1 + (verify_1024_mean - y1) * (x2 - x1) / (y2 - y1)
            
            # Annotate breakeven point
            ax.annotate(f'1024 Breakeven ≈ {breakeven_1024:.0f}',
                       xy=(breakeven_1024, verify_1024_mean),
                       xytext=(breakeven_1024 * 0.35, verify_1024_mean * 1.15),
                       fontsize=9, color='#06A77D',
                       arrowprops=dict(arrowstyle='->', color='#06A77D', lw=1.5),
                       bbox=dict(boxstyle='round,pad=0.3', facecolor='white', 
                                edgecolor='#06A77D', alpha=0.9))
            break
    
    # Labels and title
    ax.set_xlabel('Number of Indices Verified', fontweight='bold')
    ax.set_ylabel('Time (µs)', fontweight='bold')
    
    # Use log scale for x-axis to better show the relationship
    ax.set_xscale('log', base=2)
    # Use all unique indices from both datasets
    all_indices = sorted(set(indices_512.tolist() + indices_1024.tolist()))
    ax.set_xticks(all_indices)
    ax.set_xticklabels(all_indices)
    
    # Calculate y-axis limits: start from 0, end slightly above max data for tighter fit
    max_y = max(means_512.max(), means_1024.max(), verify_512_mean, verify_1024_mean) * 1.1  # 10% padding above highest point
    
    # Add y-axis ticks at the verify baselines for easy reference
    yticks = list(ax.get_yticks())
    yticks.append(verify_512_mean)
    yticks.append(verify_1024_mean)
    # Filter ticks to only include those within our desired range
    yticks = [y for y in yticks if 0 <= y <= max_y]
    yticks = sorted(set(yticks))  # Remove duplicates and sort
    ax.set_yticks(yticks)
    
    # Set y-axis limits AFTER setting ticks to prevent auto-expansion
    ax.set_ylim(bottom=0, top=max_y)
    
    # Legend with border in bottom right corner
    ax.legend(loc='lower right', frameon=True, edgecolor='black', fancybox=False)
    
    # Add grid
    ax.grid(True, alpha=0.3, linestyle='-')
    ax.set_axisbelow(True)
    
    plt.tight_layout()
    
    # Save the figure
    output_path = Path("/Users/vamsi/Github/falcon-rust/plotting/fverify_benchmark.png")
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"\nPlot saved to: {output_path}")
    
    # Also save as PDF for publication quality
    pdf_path = output_path.with_suffix('.pdf')
    plt.savefig(pdf_path, bbox_inches='tight')
    print(f"PDF saved to: {pdf_path}")
    
    plt.show()

if __name__ == "__main__":
    main()
