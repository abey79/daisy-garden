# /// script
# dependencies = [
#   "numpy",
#   "matplotlib",
#   "scipy",
# ]
# ///

# Use `uv run plot_spectrum.py <filename>`

"""
Analyze and plot frequency spectrum of noise samples from a text file.
Useful for verifying white, pink, and red noise characteristics.
"""

import numpy as np
import matplotlib.pyplot as plt
from scipy import signal
from scipy.optimize import curve_fit
import argparse
import sys


def load_samples(filename):
    """Load u16 samples from text file, one per line."""
    try:
        # Load as integers
        samples = np.loadtxt(filename, dtype=np.uint16)
        print(f"Loaded {len(samples)} samples from {filename}")

        # Convert to normalized float [-1, 1]
        # u16 range is 0-65535, center at 32768
        normalized = (samples.astype(np.float32) - 32768) / 32768

        return normalized
    except Exception as e:
        print(f"Error loading file: {e}")
        sys.exit(1)


def compute_spectrum(samples, sample_rate=44100, method='welch'):
    """Compute power spectral density of the signal."""

    if method == 'welch':
        # Welch's method - better for noise analysis
        # Uses overlapping windows for smoother estimate
        nperseg = min(len(samples) // 8, 4096)  # Window size
        freqs, psd = signal.welch(samples,
                                  fs=sample_rate,
                                  nperseg=nperseg,
                                  noverlap=nperseg // 2,
                                  scaling='density',
                                  window='hann')
    else:
        # Simple FFT method
        fft = np.fft.rfft(samples)
        psd = np.abs(fft) ** 2 / len(samples)
        freqs = np.fft.rfftfreq(len(samples), 1 / sample_rate)

    return freqs, psd


def fit_spectrum_slope(freqs, psd, fmin=20, fmax=None):
    """Fit a power law to the spectrum to determine noise color."""
    if fmax is None:
        fmax = freqs[-1] / 2  # Nyquist / 2

    # Select frequency range for fitting
    mask = (freqs >= fmin) & (freqs <= fmax) & (freqs > 0) & (psd > 0)
    f_fit = freqs[mask]
    psd_fit = psd[mask]

    # Fit in log-log space: log(PSD) = log(a) - b*log(f)
    def power_law(log_f, log_a, b):
        return log_a - b * log_f

    try:
        popt, _ = curve_fit(power_law, np.log10(f_fit), np.log10(psd_fit))
        slope = -popt[1]  # Negative because we defined as -b

        # Generate fit line
        fit_psd = 10 ** power_law(np.log10(f_fit), *popt)

        return slope, f_fit, fit_psd
    except:
        return None, None, None


def identify_noise_type(slope):
    """Identify noise type based on spectral slope."""
    if slope is None:
        return "Unknown"

    # Tolerance for classification
    tol = 0.3

    # For PSD ∝ f^(slope), the dB/octave is slope * 10 * log10(2) ≈ slope * 3.0103
    db_per_octave = slope * 3.0103

    # Classification based on slope value
    if abs(slope - 2) < tol:
        return f"Violet noise (+6 dB/octave, slope = {slope:.2f})"
    elif abs(slope - 1) < tol:
        return f"Blue noise (+3 dB/octave, slope = {slope:.2f})"
    elif abs(slope) < tol:
        return f"White noise (0 dB/octave, slope = {slope:.2f})"
    elif abs(slope + 1) < tol:
        return f"Pink noise (-3 dB/octave, slope = {slope:.2f})"
    elif abs(slope + 2) < tol:
        return f"Red/Brown noise (-6 dB/octave, slope = {slope:.2f})"
    else:
        return f"Other colored noise ({db_per_octave:.1f} dB/octave, slope = {slope:.2f})"


def plot_spectrum(freqs, psd, title="Noise Spectrum Analysis",
                  sample_rate=44100, save_path=None):
    """Create comprehensive spectrum plot."""

    # Create figure with subplots
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 10))

    # Top plot: Linear scale PSD
    ax1.semilogy(freqs, psd, 'b-', alpha=0.7, linewidth=0.5)
    ax1.set_xlabel('Frequency (Hz)')
    ax1.set_ylabel('Power Spectral Density')
    ax1.set_title(f'{title} - Linear Frequency Scale')
    ax1.grid(True, alpha=0.3)
    ax1.set_xlim([0, sample_rate / 2])

    # Bottom plot: Log-log scale with slope fitting
    mask = (freqs > 0) & (psd > 0)  # Can't plot zero on log scale
    ax2.loglog(freqs[mask], psd[mask], 'b-', alpha=0.7, linewidth=0.5,
               label='Measured spectrum')

    # Fit and plot slope
    slope, f_fit, psd_fit = fit_spectrum_slope(freqs, psd)
    if slope is not None:
        db_per_octave = slope * 3.0103
        ax2.loglog(f_fit, psd_fit, 'r--', linewidth=2,
                   label=f'Fit: slope = {slope:.2f} ({db_per_octave:.1f} dB/oct)')

        # Add reference lines for different noise types
        f_ref = np.logspace(1, np.log10(sample_rate / 2), 100)
        psd_white = np.ones_like(f_ref) * np.median(psd[mask])
        psd_pink = psd_white[0] * (f_ref / f_ref[0]) ** -1
        psd_red = psd_white[0] * (f_ref / f_ref[0]) ** -2

        ax2.loglog(f_ref, psd_white, 'g:', alpha=0.5, label='White (0 dB/oct)')
        ax2.loglog(f_ref, psd_pink, 'm:', alpha=0.5, label='Pink (-3 dB/oct)')
        ax2.loglog(f_ref, psd_red, 'c:', alpha=0.5, label='Red (-6 dB/oct)')

    ax2.set_xlabel('Frequency (Hz)')
    ax2.set_ylabel('Power Spectral Density')
    ax2.set_title(f'{title} - Log-Log Scale')
    ax2.grid(True, which="both", ls="-", alpha=0.2)
    ax2.legend()

    # Add noise type identification
    noise_type = identify_noise_type(slope)
    fig.suptitle(f'Detected: {noise_type}', fontsize=14, fontweight='bold')

    plt.tight_layout()

    if save_path:
        plt.savefig(save_path, dpi=150, bbox_inches='tight')
        print(f"Plot saved to {save_path}")

    plt.show()


def plot_time_series(samples, sample_rate=44100, num_samples=1000):
    """Plot a portion of the time series."""
    plt.figure(figsize=(10, 4))

    time = np.arange(num_samples) / sample_rate * 1000  # Convert to ms
    plt.plot(time, samples[:num_samples], 'b-', linewidth=0.5)
    plt.xlabel('Time (ms)')
    plt.ylabel('Amplitude')
    plt.title('Time Domain Signal')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.show()


def analyze_statistics(samples):
    """Print basic statistics about the samples."""
    print("\n=== Sample Statistics ===")
    print(f"Mean: {np.mean(samples):.6f} (should be near 0)")
    print(f"Std Dev: {np.std(samples):.6f}")
    print(f"Min: {np.min(samples):.6f}")
    print(f"Max: {np.max(samples):.6f}")
    print(f"RMS: {np.sqrt(np.mean(samples ** 2)):.6f}")

    # Check for clipping
    clip_threshold = 0.999
    clipped = np.sum(np.abs(samples) > clip_threshold)
    if clipped > 0:
        print(f"WARNING: {clipped} samples ({100 * clipped / len(samples):.2f}%) may be clipped!")


def main():
    parser = argparse.ArgumentParser(description='Analyze noise spectrum from sample file')
    parser.add_argument('filename', help='Text file with u16 samples (one per line)')
    parser.add_argument('-r', '--rate', type=float, default=44100,
                        help='Sample rate in Hz (default: 44100)')
    parser.add_argument('-s', '--save', help='Save plot to file')
    parser.add_argument('-t', '--time', action='store_true',
                        help='Also show time domain plot')
    parser.add_argument('-m', '--method', choices=['welch', 'fft'], default='welch',
                        help='Spectrum estimation method (default: welch)')

    args = parser.parse_args()

    # Load samples
    samples = load_samples(args.filename)

    # Show statistics
    analyze_statistics(samples)

    # Compute spectrum
    print(f"\nComputing spectrum using {args.method} method...")
    freqs, psd = compute_spectrum(samples, args.rate, method=args.method)

    # Show time series if requested
    if args.time:
        plot_time_series(samples, args.rate)

    # Plot spectrum
    plot_spectrum(freqs, psd,
                  title=f"Noise Spectrum: {args.filename}",
                  sample_rate=args.rate,
                  save_path=args.save)


if __name__ == "__main__":
    main()
