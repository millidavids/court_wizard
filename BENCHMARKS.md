# Court Wizard — Hardware Benchmarks

Performance data gathered from the `bench-release` profile (`./build_native.sh --benchmarking`) using the in-game F4 diagnostics logger.

## Test Methodology

**Scenario:** Roguelite level with maximum modifiers, infinite mana debug, walls of fire spam for particle/entity stress.

**Metric target:** 60 FPS smooth, 30 FPS acceptable floor.

**Machine under test (reference):**
- CPU: AMD Ryzen 9 9950X3D (16-core)
- GPU: Discrete + integrated Radeon (RDNA 2 iGPU)
- RAM: 32 GB
- OS: Windows 11 Pro

Constrained configurations simulated weaker hardware by pinning CPU affinity and forcing the integrated GPU via `WGPU_POWER_PREF=low`.

## Results (2026-04-14)

| Config | CPU cores | GPU | 60 FPS ceiling | Notes |
|---|---|---|---|---|
| Baseline | 8 | Discrete | ~35k entities | Smooth under peak stress; CPU ~25% utilization |
| 2-core | 2 | Discrete | ~25k entities | Playable in normal scenes; dips to 11–22 FPS at 40k+ |
| 1-core | 1 | Discrete | Never stable | Single-thread saturated; 15–25 FPS with spikes, unplayable |
| Integrated GPU | 8 | Integrated Radeon | ~32k entities @ 30 FPS | GPU-bound; CPU had 70%+ headroom |

**Memory usage:** Never exceeded ~400 MB process RSS across any configuration.

**Conclusion:** The game is overwhelmingly CPU-bound on the simulation path and GPU-bound only on the weakest integrated graphics. RAM is a non-factor for min spec.

## Published Hardware Requirements

### Minimum

| Component | Spec |
|---|---|
| **OS** | Windows 10 64-bit (or Linux/macOS equivalent) |
| **CPU** | Dual-core 2.0 GHz |
| **RAM** | 2 GB |
| **GPU** | Integrated graphics with Vulkan or DX12 support (Intel Iris Xe, AMD Vega / Radeon iGPU, or equivalent) |
| **Storage** | 500 MB |
| **Target** | 30 FPS in normal play; may dip under peak roguelite stress |

### Recommended

| Component | Spec |
|---|---|
| **OS** | Windows 10/11 64-bit |
| **CPU** | Quad-core 2.5 GHz |
| **RAM** | 4 GB |
| **GPU** | Any dedicated GPU from the last ~5 years |
| **Storage** | 500 MB |
| **Target** | Stable 60 FPS under all gameplay conditions |

## Running a Benchmark

1. Build: `./build_native.sh windows --benchmarking` (or drop `windows` for host)
2. Launch from a terminal so stdout is visible:
   ```
   ./target/x86_64-pc-windows-gnu/bench-release/court_wizard.exe 2>&1 | tee bench.log
   ```
3. Start the stress scenario, press **F4** to begin logging (samples every 2 real seconds).
4. Press **F4** again to stop. Search the log with `grep BENCH bench.log`.

### Simulating Weaker Hardware (Windows / PowerShell)

```powershell
# 2-core affinity
cmd /c "start /affinity 3 .\court_wizard.exe"

# 1-core affinity
cmd /c "start /affinity 1 .\court_wizard.exe"

# Force integrated GPU
$env:WGPU_POWER_PREF = "low"
.\court_wizard.exe

# Force DX12 / Vulkan backend
$env:WGPU_BACKEND = "dx12"
.\court_wizard.exe
```
