<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import { api, type Report } from '$lib/api';
  import { getCurrentLocation, getProfileLocationCoordinates } from '$lib/utils/geolocation';
  import 'leaflet/dist/leaflet.css';
  import 'maplibre-gl/dist/maplibre-gl.css';

  let mapElement: HTMLElement;
  let map: any;
  let L: any;
  let reports = $state<Report[]>([]);
  let loading = $state(true);
  let error = $state('');

  let lastFetchedCenter: { lat: number; lng: number } | null = null;
  let lastFetchedRadius = 0;
  let fetchTimeout: ReturnType<typeof setTimeout> | null = null;
  let isFetchingBackground = $state(false);
  let markerMap = new Map<string, any>();
  let reportDataMap = new Map<string, Report>();

  const CACHE_KEY_POS = 'lp_last_pos';

  onMount(async () => {
    if (browser) {
      const leafletModule = await import('leaflet');
      L = leafletModule.default;
      window.L = L; // Make Leaflet global for the plugin
      
      const maplibreModule = await import('maplibre-gl');
      window.maplibregl = maplibreModule.default; // Plugin needs this global
      
      await import('@maplibre/maplibre-gl-leaflet');

      // Fix Leaflet's default icon path issues
      delete (L.Icon.Default.prototype as any)._getIconUrl;
      L.Icon.Default.mergeOptions({
        iconRetinaUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-icon-2x.png',
        iconUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-icon.png',
        shadowUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-shadow.png',
      });

      await initMap();
    }
  });

  onDestroy(() => {
    if (map) {
      map.remove();
    }
    if (fetchTimeout) {
      clearTimeout(fetchTimeout);
    }
  });

  function getCachedPos() {
    if (!browser) return null;
    try {
      const val = localStorage.getItem(CACHE_KEY_POS);
      return val ? JSON.parse(val) : null;
    } catch { return null; }
  }

  async function getInitialPosition() {
    // 0. Try cached location for instant load
    const cached = getCachedPos();
    if (cached) return cached;

    // 1. Try GPS
    const coords = await getCurrentLocation({ timeout: 5000 });
    if (coords.accuracy) return { lat: coords.lat, lng: coords.lng };

    // 2. Try Profile Location (Fallback if GPS failed/denied)
    const profileCoords = await getProfileLocationCoordinates($auth.user);
    if (profileCoords) return { lat: profileCoords.lat, lng: profileCoords.lng };

    // 3. Fallback London
    return { lat: 51.5074, lng: -0.1278 };
  }

  function getZoomLevel(radiusKm: number) {
    if (radiusKm <= 1) return 14;
    if (radiusKm <= 2) return 13;
    if (radiusKm <= 5) return 12;
    if (radiusKm <= 10) return 11;
    if (radiusKm <= 20) return 10;
    if (radiusKm <= 50) return 9;
    return 8;
  }

  async function initMap() {
    if (!mapElement || map) return;

    loading = true;
    const pos = await getInitialPosition();
    const radius = $auth.user?.search_radius_km || 10;
    const zoom = getZoomLevel(radius);

    map = L.map(mapElement).setView([pos.lat, pos.lng], zoom);

    if (typeof (L as any).maplibreGL === 'function') {
      (L as any).maplibreGL({
        style: 'https://tiles.openfreemap.org/styles/bright',
        attribution: '&copy; <a href="https://openfreemap.org/">OpenFreeMap</a> contributors'
      }).addTo(map);
    } else {
      L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
      }).addTo(map);
    }

    // Force map to recalculate container size
    setTimeout(() => {
      map.invalidateSize();
    }, 100);

    // Initial fetch
    await loadReports(pos.lat, pos.lng, radius);

    // Background GPS refresh if we used a fallback/cache
    if (browser && navigator.geolocation) {
      getCurrentLocation({ enableHighAccuracy: true, timeout: 10000 }).then(coords => {
        if (coords.accuracy && map) {
          const dist = getDistanceKm(pos.lat, pos.lng, coords.lat, coords.lng);
          if (dist > 0.1) { // > 100m difference
            map.panTo([coords.lat, coords.lng]);
            loadReports(coords.lat, coords.lng, radius, true);
          }
          localStorage.setItem(CACHE_KEY_POS, JSON.stringify({ lat: coords.lat, lng: coords.lng }));
        }
      });
    }

    // Watch for movement
    map.on('moveend', () => {
      handleMapMove();
    });
  }

  function handleMapMove() {
    if (!map) return;
    
    if (fetchTimeout) clearTimeout(fetchTimeout);
    
    fetchTimeout = setTimeout(() => {
      const center = map.getCenter();
      const radius = Math.min(getRadiusFromView(), 50); // Cap at 50km
      
      // Optimization: Only fetch if we've moved significantly
      // (more than 30% of the last fetched radius)
      if (lastFetchedCenter) {
        const dist = getDistanceKm(center.lat, center.lng, lastFetchedCenter.lat, lastFetchedCenter.lng);
        if (dist < lastFetchedRadius * 0.3) {
            return;
        }
      }
      
      loadReports(center.lat, center.lng, radius, true);
    }, 500);
  }

  function getRadiusFromView() {
    if (!map) return 10;
    const bounds = map.getBounds();
    const center = map.getCenter();
    const northEast = bounds.getNorthEast();
    // Distance from center to corner in km
    return getDistanceKm(center.lat, center.lng, northEast.lat, northEast.lng);
  }

  function getDistanceKm(lat1: number, lon1: number, lat2: number, lon2: number) {
    const R = 6371;
    const dLat = (lat2 - lat1) * Math.PI / 180;
    const dLon = (lon2 - lon1) * Math.PI / 180;
    const a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
              Math.cos(lat1 * Math.PI / 180) * Math.cos(lat2 * Math.PI / 180) *
              Math.sin(dLon / 2) * Math.sin(dLon / 2);
    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
    return R * c;
  }

  async function recenter() {
    isFetchingBackground = true;
    try {
      const coords = await getCurrentLocation({ enableHighAccuracy: true, timeout: 10000 });
      if (map) {
        map.setView([coords.lat, coords.lng], map.getZoom());
        const radius = getRadiusFromView();
        loadReports(coords.lat, coords.lng, radius, true);
        localStorage.setItem(CACHE_KEY_POS, JSON.stringify({ lat: coords.lat, lng: coords.lng }));
      }
    } catch (e) {
      error = "Could not find your location";
    } finally {
      isFetchingBackground = false;
    }
  }

  async function loadReports(lat: number, lng: number, radius: number, isBackground = false) {
    if (!$auth.token) return;
    
    if (isBackground) isFetchingBackground = true;
    else loading = true;

    try {
      // Fetch buffered radius
      const fetchRadius = radius * 1.5;
      const data = await api.reports.getNearby(lat, lng, fetchRadius, $auth.token);
      
      // Merge into local cache
      data.forEach(r => reportDataMap.set(r.id, r));
      reports = Array.from(reportDataMap.values());
      
      lastFetchedCenter = { lat, lng };
      lastFetchedRadius = fetchRadius;
      
      updateMarkers(data);
    } catch (e: any) {
      error = e.message || 'Failed to load markers';
    } finally {
      loading = false;
      isFetchingBackground = false;
    }
  }

  function updateMarkers(nearbyReports: Report[]) {
    if (!map || !L) return;

    nearbyReports.forEach(report => {
      // Skip if marker already exists
      if (markerMap.has(report.id)) return;

      const marker = L.marker([report.latitude, report.longitude])
        .addTo(map);

      // Hover popup with photo
      const popupContent = `
        <div class="p-1 max-w-[200px]">
          ${report.photo_before ? `<img src="${report.photo_before}" class="w-full h-32 object-cover rounded mb-2" loading="lazy" />` : '<div class="w-full h-32 bg-slate-100 flex items-center justify-center rounded mb-2">📸</div>'}
          <p class="text-xs text-slate-600 line-clamp-2">${report.description || 'No description'}</p>
          <p class="text-[10px] text-slate-400 mt-1">Click to view details</p>
        </div>
      `;

      marker.bindPopup(popupContent, {
        closeButton: false,
        offset: [0, -20],
        className: 'report-popup'
      });

      marker.on('mouseover', function (this: any) {
        this.openPopup();
      });

      marker.on('mouseout', function (this: any) {
        this.closePopup();
      });

      marker.on('click', () => {
        goto(`/app/report/${report.id}`);
      });

      markerMap.set(report.id, marker);
    });
  }
</script>

<div class="h-[calc(100vh-64px)] w-full relative">
  <div bind:this={mapElement} class="h-full w-full z-0 bg-slate-50"></div>

  {#if loading && reports.length === 0}
    <div class="absolute inset-0 bg-white/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div class="flex flex-col items-center">
        <div class="w-12 h-12 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
        <p class="mt-4 text-slate-600 font-medium">Loading Map...</p>
      </div>
    </div>
  {/if}

  {#if isFetchingBackground}
    <div class="absolute top-4 right-16 z-10 bg-white/90 backdrop-blur-sm px-3 py-1.5 rounded-full shadow-md border border-slate-200 flex items-center gap-2">
      <div class="w-3 h-3 border-2 border-primary-200 border-t-primary-600 rounded-full animate-spin text-[8px]"></div>
      <span class="text-[10px] font-bold text-slate-700 uppercase tracking-tighter">Refreshing</span>
    </div>
  {/if}

  <!-- Map Controls -->
  <div class="absolute top-4 right-4 z-10 flex flex-col gap-2">
    <button 
      onclick={recenter}
      class="bg-white p-2.5 rounded-lg shadow-lg border border-slate-200 hover:bg-slate-50 transition-colors active:scale-95"
      title="Recenter to my location"
    >
      📍
    </button>
  </div>

  {#if error}
    <div class="absolute top-4 left-1/2 -translate-x-1/2 z-50">
        <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-2 rounded-md shadow-md text-sm font-medium">
            {error}
        </div>
    </div>
  {/if}

  <!-- Legend / Info -->
  <div class="absolute bottom-8 right-8 z-10 bg-white p-3 rounded-lg shadow-lg border border-slate-200 max-w-xs">
    <h4 class="text-xs font-bold text-slate-900 mb-2 uppercase tracking-wider">Map Explorer</h4>
    <p class="text-xs text-slate-600">
      Showing reports within your area. Pan and zoom to discover more.
    </p>
    <div class="mt-3 flex items-center gap-2">
        <div class="w-3 h-3 bg-primary-500 rounded-full"></div>
        <span class="text-[10px] text-slate-500">Litter Reports</span>
    </div>
  </div>
</div>

<style>
  :global(.report-popup .leaflet-popup-content-wrapper) {
    padding: 0;
    overflow: hidden;
    border-radius: 12px;
    box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
  }
  :global(.report-popup .leaflet-popup-content) {
    margin: 0;
  }
  :global(.leaflet-marker-icon) {
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }
  :global(.leaflet-control-attribution) {
    font-size: 9px !important;
    background: rgba(255, 255, 255, 0.7) !important;
    backdrop-filter: blur(2px);
  }
</style>
