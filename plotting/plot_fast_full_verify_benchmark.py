#!/usr/bin/env python3
"""
Visualization of Falcon Stream Verification Benchmark Results

This script plots the performance of processing a stream of signatures (valid + invalid)
using two approaches:
1. Baseline: Verify all signatures fully.
2. Optimized: Fast Verify (fverify) first, then Full Verify if passed.

It generates plots showing:
- Total execution time vs Invalid Signature Fraction
- Speedup factor vs Invalid Signature Fraction
"""

import json
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
import sys

# Configure matplotlib
# Configure matplotlib for publication-quality figures
plt.style.use('seaborn-v0_8-whitegrid')
plt.rcParams.update({
    'font.size': 12,
    'axes.labelsize': 14,
    'axes.titlesize': 16,
    'legend.fontsize': 12,
    'figure.figsize': (12, 8),
    'figure.dpi': 150,
})

def load_criterion_estimate(path: Path) -> dict:
    """Load timing estimates from Criterion's JSON output."""
    estimates_path = path / "new" / "estimates.json"
    if not estimates_path.exists():
        estimates_path = path / "base" / "estimates.json"
    
    if not estimates_path.exists():
        return None
        
    with open(estimates_path) as f:
        data = json.load(f)
    
    # Extract mean timing (in nanoseconds)
    mean = data["mean"]["point_estimate"]
    return {"mean": mean}

def main():
    print("Starting benchmark plot generation...", flush=True)
    criterion_dir = Path("/Users/vamsi/Github/falcon-rust/target/criterion")
    base_dir = criterion_dir / "stream_verification"
    
    if not base_dir.exists():
        print(f"Error: Benchmark directory not found: {base_dir}")
        print("Please run 'cargo bench --bench fast_full_verify'")
        sys.exit(1)

    variants = ["falcon512", "falcon1024"]
    index_counts = [1, 8]
    # Match the Rust formatting {:.2}
    invalid_fractions = [0.01, 0.1, 0.5, 0.9, 0.99]
    
    results = {v: {} for v in variants}
    
    # Load data
    for variant in variants:
        # Load Baseline
        results[variant]["baseline"] = []
        for frac in invalid_fractions:
            # Rust format: invalid_{:.2} (Criterion flattens / to _)
            subdir_name = f"{variant}_baseline_invalid_{frac:.2f}"
            path = base_dir / subdir_name
            data = load_criterion_estimate(path)
            if data:
                # Store mean time in milliseconds (ms) for 1000 sigs
                results[variant]["baseline"].append(data["mean"] / 1_000_000)
            else:
                print(f"Warning: Missing data for {subdir_name}", flush=True)
                results[variant]["baseline"].append(None)
                
        # Load Optimized (per index count)
        results[variant]["optimized"] = {idx: [] for idx in index_counts}
        for idx in index_counts:
            for frac in invalid_fractions:
                subdir_name = f"{variant}_indices_{idx}_invalid_{frac:.2f}"
                path = base_dir / subdir_name
                data = load_criterion_estimate(path)
                if data:
                    results[variant]["optimized"][idx].append(data["mean"] / 1_000_000)
                else:
                    print(f"Warning: Missing data for {subdir_name}", flush=True)
                    results[variant]["optimized"][idx].append(None)

    # Plotting
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Colors for different variants and index counts
    # Falcon-512: Blues/Cyans
    # Falcon-1024: Reds/Oranges
    colors_512 = ['#0077B6', '#48CAE4'] # Strong Blue, Cyan
    colors_1024 = ['#9D0208', '#E85D04'] # Deep Red, Orange
    
    baseline_medians = []
    
    for i, variant in enumerate(variants):
        variant_name = "Falcon-512" if variant == "falcon512" else "Falcon-1024"
        
        # Prepare baseline data
        baseline_raw = [x if x is not None else np.nan for x in results[variant]["baseline"]]
        # Calculate median for flat line
        baseline_median = np.nanmedian(baseline_raw)
        baseline_medians.append(baseline_median)
        
        # Plot Baseline as flat line across the range
        # Use simple dashed lines for baselines
        base_color = 'navy' if variant == 'falcon512' else 'darkred'
        base_style = '--' if variant == 'falcon512' else '-.'
        
        # Plot Baseline as flat line across the entire width
        # Use simple dashed lines for baselines
        base_color = 'navy' if variant == 'falcon512' else 'darkred'
        base_style = '--' if variant == 'falcon512' else '-.'
        
        ax.axhline(y=baseline_median, linestyle=base_style, color=base_color, 
                   linewidth=2.5, alpha=0.8, label=f'{variant_name} Baseline',
                   zorder=2)
        
        # Plot Optimized lines
        current_colors = colors_512 if variant == 'falcon512' else colors_1024
        marker_style = 'o' if variant == 'falcon512' else 's'
        
        for j, idx in enumerate(index_counts):
            opt_times = [x if x is not None else np.nan for x in results[variant]["optimized"][idx]]
            
            c = current_colors[j % len(current_colors)]
            ax.plot(invalid_fractions, opt_times, 
                    marker=marker_style, markersize=10, linewidth=3, color=c,
                    label=f'{variant_name} Fast Verify ({idx} indices)',
                    zorder=3)
    
    ax.set_xlabel('Fraction of Invalid Signatures', fontweight='bold')
    ax.set_ylabel('Time Taken to verify 1000 Signatures (ms)', fontweight='bold')
    
    # Set x-limits to perfectly fit the 0-1 range without gaps
    ax.set_xlim(0, 1)
    ax.set_xticks(invalid_fractions)
    ax.set_xticklabels([f'{x:.2f}' for x in invalid_fractions], rotation=0)
    
    # Legend with border matching fverify plot style, positioned in the gap between baselines
    ax.legend(fontsize=12, loc='center right', bbox_to_anchor=(0.99, 0.65), 
              frameon=True, edgecolor='black', fancybox=False)
    
    # Add y-axis ticks at the baseline medians
    yticks = list(ax.get_yticks())
    for bm in baseline_medians:
        yticks.append(bm)
    
    # Filter ticks and set
    max_y = ax.get_ylim()[1]
    yticks = [y for y in yticks if 0 <= y <= max_y]
    yticks = sorted(set(yticks))
    ax.set_yticks(yticks)
    ax.set_yticklabels([f'{y:.1f}' for y in yticks])
    
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    
    # Save the figure
    output_path = Path("/Users/vamsi/Github/falcon-rust/plotting/stream_verify_benchmark.png")
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"\nPlot saved to: {output_path}", flush=True)
    
    # Also save as PDF
    pdf_path = output_path.with_suffix('.pdf')
    plt.savefig(pdf_path, bbox_inches='tight')
    print(f"PDF saved to: {pdf_path}", flush=True)
    
    # No plt.show()

if __name__ == "__main__":
    main()
