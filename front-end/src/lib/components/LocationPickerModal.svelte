<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { browser } from '$app/environment';
  import LocationSearch from './LocationSearch.svelte';
  import 'leaflet/dist/leaflet.css';
  import 'maplibre-gl/dist/maplibre-gl.css';

  const dispatch = createEventDispatcher<{
    close: void;
    select: { lat: number; lng: number; label: string; radius: number };
  }>();

  export let initialLat: number = 51.5074;
  export let initialLng: number = -0.1278;
  export let initialRadius: number = 10;

  let mapElement: HTMLElement;
  let map: any;
  let L: any;
  let marker: any;
  let radiusCircle: any;
  let selectedLat = initialLat;
  let selectedLng = initialLng;
  let selectedRadius = initialRadius;
  let selectedLabel = '';

  onMount(async () => {
    if (browser) {
      const leafletModule = await import('leaflet');
      L = leafletModule.default;
      
      const maplibreModule = await import('maplibre-gl');
      (window as any).maplibregl = maplibreModule.default;
      
      await import('@maplibre/maplibre-gl-leaflet');

      // Fix icons
      delete (L.Icon.Default.prototype as any)._getIconUrl;
      L.Icon.Default.mergeOptions({
        iconRetinaUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-icon-2x.png',
        iconUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-icon.png',
        shadowUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-shadow.png',
      });

      initMap();
    }
  });

  function initMap() {
    map = L.map(mapElement).setView([initialLat, initialLng], 13);

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

    marker = L.marker([initialLat, initialLng], { draggable: true }).addTo(map);
    radiusCircle = L.circle([initialLat, initialLng], {
      radius: selectedRadius * 1000,
      color: '#3b82f6',
      fillColor: '#3b82f6',
      fillOpacity: 0.1,
      weight: 1
    }).addTo(map);

    function updatePosition(lat: number, lng: number) {
      selectedLat = lat;
      selectedLng = lng;
      marker.setLatLng([lat, lng]);
      radiusCircle.setLatLng([lat, lng]);
      selectedLabel = `Custom location (${lat.toFixed(4)}, ${lng.toFixed(4)})`;
    }

    marker.on('dragend', (e: any) => {
      const { lat, lng } = e.target.getLatLng();
      updatePosition(lat, lng);
    });

    map.on('click', (e: any) => {
      const { lat, lng } = e.latlng;
      updatePosition(lat, lng);
    });

    // Initial radius set
    radiusCircle.setRadius(selectedRadius * 1000);
    
    // Force redraw
    setTimeout(() => {
        map.invalidateSize();
    }, 100);
  }

  function handleSearchSelect(event: CustomEvent) {
    const { lat, lng, label } = event.detail;
    if (lat && lng) {
      selectedLat = lat;
      selectedLng = lng;
      selectedLabel = label;
      
      if (map && marker) {
        map.setView([lat, lng], 13);
        marker.setLatLng([lat, lng]);
        radiusCircle.setLatLng([lat, lng]);
      }
    }
  }

  function confirmSelection() {
    dispatch('select', {
      lat: selectedLat,
      lng: selectedLng,
      label: selectedLabel || 'Selected Location',
      radius: selectedRadius
    });
    dispatch('close');
  }
</script>

<div class="fixed inset-0 z-50 overflow-y-auto" aria-labelledby="modal-title" role="dialog" aria-modal="true">
  <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
    <!-- Backdrop -->
    <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" aria-hidden="true" on:click={() => dispatch('close')}></div>

    <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>

    <div class="relative inline-block align-bottom bg-white rounded-lg px-4 pt-5 pb-4 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full sm:p-6">
      <div class="sm:flex sm:items-start">
        <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left w-full">
          <h3 class="text-lg leading-6 font-medium text-gray-900" id="modal-title">
            Change Viewing Location
          </h3>
          <div class="mt-2">
            <p class="text-sm text-gray-500 mb-4">
              Select a location to see litter reports in that area. This won't change your profile location permanently.
            </p>
            
            <div class="mb-4">
                <LocationSearch on:select={handleSearchSelect} />
            </div>

            <div class="h-64 w-full rounded-lg overflow-hidden border border-gray-300 relative bg-slate-100">
                <div bind:this={mapElement} class="h-full w-full z-0"></div>
            </div>
            <div class="mt-4">
                <label for="modalRadius" class="flex items-center justify-between text-xs font-medium text-gray-500 mb-1">
                    <span>Search Radius</span>
                    <span class="text-primary-600 font-bold">{selectedRadius} km</span>
                </label>
                <input 
                    id="modalRadius"
                    type="range" 
                    min="1" 
                    max="100" 
                    bind:value={selectedRadius} 
                    on:input={() => {
                        if (radiusCircle) radiusCircle.setRadius(selectedRadius * 1000);
                    }}
                    class="w-full h-1.5 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-600"
                />
            </div>
            <p class="text-xs text-gray-400 mt-2 text-center">
                Click map to move pin. Adjust slider to set radius.
            </p>
          </div>
        </div>
      </div>
      <div class="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
        <button 
            type="button" 
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-primary-600 text-base font-medium text-white hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 sm:ml-3 sm:w-auto sm:text-sm"
            on:click={confirmSelection}
        >
          Update Feed
        </button>
        <button 
            type="button" 
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 sm:mt-0 sm:w-auto sm:text-sm"
            on:click={() => dispatch('close')}
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</div>
