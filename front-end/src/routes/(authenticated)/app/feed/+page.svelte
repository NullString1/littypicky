<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type Report } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { getCurrentLocation, calculateDistance, getProfileLocationCoordinates } from '$lib/utils/geolocation';
  import { getStatusColor } from '$lib/utils/status';
  import { formatDateShort } from '$lib/utils/date';
  import LocationPickerModal from '$lib/components/LocationPickerModal.svelte';

  let reports = $state<Report[]>([]);
  let loading = $state(true);
  let error = $state('');
  let userLocation = $state<{ lat: number; lng: number } | null>(null);
  let searchRadius = $state(10);
  let showLocationModal = $state(false);
  let locationOverrideLabel = $state('');

  // Use Svelte 5 $derived for automatic memoization
  // Only recalculates when reports or userLocation changes
  let reportsWithDistance = $derived(
    userLocation 
      ? reports.map(r => {
          const loc = userLocation!; // Non-null assertion since we're in the truthy branch
          return {
            ...r,
            distance: calculateDistance(loc.lat, loc.lng, r.latitude, r.longitude)
          };
        })
      : reports.map(r => ({ ...r, distance: null }))
  );

  // Helper to open Google Maps
  function openDirections(lat: number, lng: number) {
    window.open(`https://www.google.com/maps/dir/?api=1&destination=${lat},${lng}`, '_blank');
  }

  async function loadReports() {
    loading = true;
    error = '';
    try {
        if (!$auth.token) return;

        const { lat, lng } = userLocation || { lat: 51.5074, lng: -0.1278 };
        const data = await api.reports.getNearby(lat, lng, searchRadius, $auth.token);
        reports = data;
    } catch (e: any) {
        error = e.message || 'Failed to load reports';
    } finally {
        loading = false;
    }
  }

  onMount(async () => {
    // 1. Try to get current location
    const coords = await getCurrentLocation();
    
    // 2. If accurate, use it
    if (coords.accuracy) {
        userLocation = coords;
    } else {
        // 3. If failed, try profile location
        const profileCoords = await getProfileLocationCoordinates($auth.user);
        if (profileCoords) {
            userLocation = profileCoords;
        } else {
            // 4. Fallback to London (default from getCurrentLocation was mostly likely London anyway, but be explicit)
            userLocation = { lat: 51.5074, lng: -0.1278 };
        }
    }
    
    if ($auth.user?.search_radius_km) {
        searchRadius = $auth.user.search_radius_km;
    }
    
    loadReports();
  });
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    
    <!-- Header -->
    <div class="md:flex md:items-center md:justify-between mb-8">
      <div class="flex-1 min-w-0">
        <h2 class="text-2xl font-bold leading-7 text-slate-900 sm:text-3xl sm:truncate">
          Litter Feed
        </h2>
        <p class="mt-1 text-sm text-slate-500">
          Find a messy spot near you and clean it up! First come, first serve.
        </p>
      </div>
      <div class="mt-4 flex flex-col md:flex-row gap-2 md:mt-0 md:ml-4">
        <button 
          onclick={() => showLocationModal = true}
          class="inline-flex items-center justify-center px-4 py-2 border border-slate-300 rounded-md shadow-sm text-sm font-medium text-slate-700 bg-white hover:bg-slate-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500"
        >
          <span>📍</span>
          <span class="ml-2">{locationOverrideLabel || 'Change Location'}</span>
        </button>
        <a href="/app/report" class="inline-flex items-center justify-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500">
          Report New Spot
        </a>
      </div>
    </div>

    <!-- Search Radius Slider -->
    <div class="mb-6 bg-white p-4 rounded-lg shadow-sm border border-slate-200">
      <label for="feedRadius" class="flex items-center justify-between text-sm font-medium text-slate-700 mb-2">
        <span>Search Radius</span>
        <span class="text-primary-600 font-bold">{searchRadius} km</span>
      </label>
      <input 
        id="feedRadius"
        type="range" 
        min="1" 
        max="100" 
        bind:value={searchRadius} 
        onchange={loadReports}
        class="w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-600"
      />
      <div class="flex justify-between text-xs text-slate-400 mt-1">
        <span>1 km</span>
        <span>50 km</span>
        <span>100 km</span>
      </div>
    </div>

    {#if showLocationModal}
      <LocationPickerModal 
        initialLat={userLocation?.lat || 51.5074} 
        initialLng={userLocation?.lng || -0.1278}
        initialRadius={searchRadius}
        on:close={() => showLocationModal = false}
        on:select={(e) => {
          userLocation = { lat: e.detail.lat, lng: e.detail.lng };
          searchRadius = e.detail.radius;
          locationOverrideLabel = e.detail.label.length > 20 ? e.detail.label.substring(0, 20) + '...' : e.detail.label;
          loadReports();
        }}
      />
    {/if}

    {#if loading}
        <div class="flex justify-center py-12">
            <div class="w-12 h-12 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
        </div>
    {:else if error}
        <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6">
            {error}
        </div>
    {:else if reports.length === 0}
        <div class="text-center py-12 bg-white rounded-lg shadow border border-slate-200">
            <span class="text-4xl block mb-4">🎉</span>
            <h3 class="text-lg font-medium text-slate-900">No reports nearby!</h3>
            <p class="mt-2 text-slate-500">Great job keeping the area clean. Try increasing the search radius or check back later.</p>
        </div>
    {:else}
        <!-- Feed Grid -->
        <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {#each reportsWithDistance as report}
            <div class="bg-white overflow-hidden shadow rounded-lg border border-slate-200 flex flex-col transition hover:shadow-md">
            <!-- Photo -->
            {#if report.photo_before}
                <div class="h-48 w-full bg-slate-200 overflow-hidden relative group">
                   <img 
                     src={report.photo_before} 
                     alt="Litter report" 
                     class="w-full h-full object-cover"
                     loading="lazy"
                     decoding="async"
                   />
                   <div class="absolute inset-0 bg-black opacity-0 group-hover:opacity-10 transition-opacity pointer-events-none"></div>
                </div>
            {:else}
                <div class="h-48 w-full bg-slate-200 flex items-center justify-center text-slate-400">
                    <span class="text-4xl">📸</span>
                </div>
            {/if}
            
            <div class="px-4 py-5 sm:p-6 flex-1 flex flex-col">
                <div class="flex items-center justify-between mb-2">
                   <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(report.status)} uppercase tracking-wide`}>
                     {report.status}
                   </span>
                   {#if report.distance}
                     <span class="text-xs text-slate-500 font-medium">{report.distance} away</span>
                   {/if}
                </div>
                
                <h3 class="text-lg leading-6 font-bold text-slate-900 mb-1">
                  {report.address || `${report.latitude.toFixed(4)}, ${report.longitude.toFixed(4)}`}
                </h3>
                <p class="text-sm text-slate-500 line-clamp-2 mb-4">
                  {report.description || 'No description provided.'}
                </p>
                
                <div class="mt-auto pt-4 flex gap-2">
                    <a href={`/app/report/${report.id}`} class="flex-1 flex items-center justify-center px-4 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50">
                        View Details
                    </a>
                    <button 
                    onclick={() => openDirections(report.latitude, report.longitude)}
                    class="flex items-center justify-center px-3 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50"
                    title="Get Directions"
                    >
                        📍
                    </button>
                </div>
            </div>
            <div class="bg-slate-50 px-4 py-3 sm:px-6 border-t border-slate-100 flex justify-between items-center text-xs text-slate-500">
                <span>Reported {formatDateShort(report.created_at)}</span>
            </div>
            </div>
        {/each}
        </div>
    {/if}
  </div>
</div>
