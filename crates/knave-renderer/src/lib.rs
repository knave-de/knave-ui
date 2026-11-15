//! wgpu-backed rendering boundary for Knave UI scenes.

use std::{error::Error, fmt};

mod images;
mod opacity;
mod painter;
mod shapes;
mod text;
pub use painter::{PainterError, RendererStats, WgpuPainter};

pub use knave_ui::{DisplayCommand, DisplayList, RenderCommand, RenderList};

/// Selecting a GPU adapter failed.
#[derive(Debug)]
pub struct AdapterSelectionError(String);

impl fmt::Display for AdapterSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Error for AdapterSelectionError {}

/// Select a rendering adapter deterministically, including CPU software adapters.
///
/// On hybrid laptops (integrated plus discrete GPU) an unconstrained
/// [`wgpu::Instance::request_adapter`] can pick the discrete GPU even though the
/// compositor composites on the integrated one; every frame then crosses GPUs
/// over PRIME and can stall the whole desktop. Selection therefore:
///
/// - honors `WGPU_ADAPTER_NAME`, `WGPU_BACKEND`, and `WGPU_POWER_PREF`; and
/// - defaults to [`wgpu::PowerPreference::LowPower`], preferring the
///   integrated GPU when one is present; and
/// - uses a compatible CPU software adapter (such as Mesa llvmpipe or lavapipe)
///   when no hardware adapter is available.
///
/// Ties keep enumeration order. The returned adapter reports its choice through
/// [`wgpu::Adapter::get_info`]; callers should log it.
pub async fn select_adapter(
    instance: &wgpu::Instance,
    compatible_surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, AdapterSelectionError> {
    let power = wgpu::PowerPreference::from_env().unwrap_or(wgpu::PowerPreference::LowPower);
    let backends = wgpu::Backends::from_env().unwrap_or_default();
    let name = std::env::var("WGPU_ADAPTER_NAME")
        .ok()
        .map(|name| name.to_lowercase());

    // Try the user's requested/normal backends first. If they have no
    // compatible adapter, retry all compiled backends so Mesa software GL or
    // Vulkan drivers can provide CPU rendering even with a restrictive
    // WGPU_BACKEND setting.
    let mut candidates = instance.enumerate_adapters(backends).await;
    let requested_has_eligible = candidates.iter().any(|adapter| {
        let surface_matches =
            compatible_surface.is_none_or(|surface| adapter.is_surface_supported(surface));
        let name_matches = name
            .as_ref()
            .is_none_or(|name| adapter.get_info().name.to_lowercase().contains(name));
        surface_matches && name_matches
    });
    if !requested_has_eligible {
        candidates = instance.enumerate_adapters(wgpu::Backends::all()).await;
    }
    let mut best: Option<(i32, usize)> = None;
    for (index, adapter) in candidates.iter().enumerate() {
        if let Some(surface) = compatible_surface
            && !adapter.is_surface_supported(surface)
        {
            continue;
        }
        let info = adapter.get_info();
        if let Some(name) = &name
            && !info.name.to_lowercase().contains(name)
        {
            continue;
        }
        // Without an explicit adapter name, preserve the normal hardware
        // preference and consider CPU adapters only after hardware is absent.
        // A name match is an explicit request and may intentionally select CPU.
        if info.device_type == wgpu::DeviceType::Cpu && name.is_none() {
            continue;
        }
        let score = adapter_score(&info, power);
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, index));
        }
    }
    if let Some((_, index)) = best {
        return Ok(candidates.swap_remove(index));
    }

    // Prefer any CPU adapter found on the same instance as the surface.
    if let Some((_, index)) = candidates
        .iter()
        .enumerate()
        .filter(|(_, adapter)| {
            adapter.get_info().device_type == wgpu::DeviceType::Cpu
                && compatible_surface.is_none_or(|surface| adapter.is_surface_supported(surface))
        })
        .find(|(_, adapter)| {
            name.as_ref()
                .is_none_or(|name| adapter.get_info().name.to_lowercase().contains(name))
        })
        .map(|(index, adapter)| (adapter_score(&adapter.get_info(), power), index))
    {
        return Ok(candidates.swap_remove(index));
    }
    Err(AdapterSelectionError(format!(
        "no compatible adapter found for {backends:?}, including a forced software fallback; install a Mesa software driver such as llvmpipe or lavapipe"
    )))
}

fn adapter_score(info: &wgpu::AdapterInfo, preference: wgpu::PowerPreference) -> i32 {
    match (preference, info.device_type) {
        (wgpu::PowerPreference::LowPower, wgpu::DeviceType::IntegratedGpu) => 1,
        (wgpu::PowerPreference::HighPerformance, wgpu::DeviceType::DiscreteGpu) => 1,
        (wgpu::PowerPreference::LowPower, wgpu::DeviceType::Cpu) => 0,
        (wgpu::PowerPreference::HighPerformance, wgpu::DeviceType::Cpu) => -1,
        _ => 0,
    }
}

pub struct WgpuRenderer {
    instance: wgpu::Instance,
}

impl WgpuRenderer {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
        }
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }
}

impl Default for WgpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(device_type: wgpu::DeviceType) -> wgpu::AdapterInfo {
        wgpu::AdapterInfo {
            name: "test".into(),
            vendor: 0,
            device: 0,
            device_type,
            device_pci_bus_id: String::new(),
            backend: wgpu::Backend::Vulkan,
            subgroup_min_size: 0,
            subgroup_max_size: 0,
            transient_saves_memory: None,
            limit_bucket: None,
            driver: String::new(),
            driver_info: String::new(),
        }
    }

    #[test]
    fn low_power_prefers_integrated_over_discrete() {
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::IntegratedGpu),
                wgpu::PowerPreference::LowPower
            ),
            1
        );
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::DiscreteGpu),
                wgpu::PowerPreference::LowPower
            ),
            0
        );
    }

    #[test]
    fn high_power_prefers_discrete_over_integrated() {
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::DiscreteGpu),
                wgpu::PowerPreference::HighPerformance
            ),
            1
        );
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::IntegratedGpu),
                wgpu::PowerPreference::HighPerformance
            ),
            0
        );
    }

    #[test]
    fn low_power_prefers_integrated_but_accepts_cpu_fallback() {
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::IntegratedGpu),
                wgpu::PowerPreference::LowPower
            ),
            1
        );
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::Cpu),
                wgpu::PowerPreference::LowPower
            ),
            0
        );
    }

    #[test]
    fn no_preference_scores_everything_equal() {
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::DiscreteGpu),
                wgpu::PowerPreference::None
            ),
            0
        );
        assert_eq!(
            adapter_score(
                &info(wgpu::DeviceType::IntegratedGpu),
                wgpu::PowerPreference::None
            ),
            0
        );
    }
}
