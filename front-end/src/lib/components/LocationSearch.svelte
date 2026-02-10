<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { getCurrentLocation, reverseGeocode } from '$lib/utils/geolocation';

  const dispatch = createEventDispatcher<{
    select: { city: string; country: string; label: string; lat?: number; lng?: number };
  }>();

  type LocationOption = {
    label: string;
    city: string;
    country: string;
    lat?: number;
    lng?: number;
  };

  export let initialValue = '';
  export let label = 'Location';
  export let placeholder = 'Start typing your city';
  export let required = false;

  let query = initialValue;
  let locationOptions: LocationOption[] = [];
  let showDropdown = false;
  let searchStatus = '';
  let isLocating = false;
  let locationStatus = '';
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;
  let hideDropdownTimeout: ReturnType<typeof setTimeout> | null = null;

  function handleInput(event: Event) {
    const rawValue = (event.target as HTMLInputElement).value;
    query = rawValue;
    searchStatus = '';

    if (!query.trim() || query.length < 2) {
      showDropdown = false;
      return;
    }

    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      void loadLocationOptions(query);
    }, 300);
  }

  function handleFocus() {
    if (locationOptions.length) {
      showDropdown = true;
    }
  }

  function handleBlur() {
    if (hideDropdownTimeout) clearTimeout(hideDropdownTimeout);
    hideDropdownTimeout = setTimeout(() => {
      showDropdown = false;
    }, 150);
  }

  async function loadLocationOptions(q: string) {
    try {
      searchStatus = 'Searching...';
      const response = await fetch(
        `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(q)}&count=10&language=en&format=json`
      );
      const data = await response.json();
      const results = Array.isArray(data?.results) ? data.results : [];
      
      const mapped = results.map((item: any) => {
        const name = item?.name as string;
        if (!name) return null;
        const admin = item?.admin1 as string;
        const admin2 = item?.admin2 as string;
        const countryName = item?.country as string;
        const labelParts = [name, admin, admin2, countryName].filter(Boolean);
        
        return {
          label: labelParts.join(', '),
          city: admin ? `${name}, ${admin}` : name,
          country: countryName || '',
          lat: item.latitude,
          lng: item.longitude
        };
      }).filter(Boolean) as LocationOption[];

      // Deduplicate
      const uniqueMap = new Map<string, LocationOption>();
      mapped.forEach(item => {
        if (!uniqueMap.has(item.label)) {
          uniqueMap.set(item.label, item);
        }
      });

      locationOptions = Array.from(uniqueMap.values()).slice(0, 10);
      searchStatus = locationOptions.length ? '' : 'No matches found.';
      showDropdown = locationOptions.length > 0;
    } catch (err) {
      console.error('Location lookup failed:', err);
      locationOptions = [];
      searchStatus = 'No matches found.';
      showDropdown = false;
    }
  }

  function handleSelect(option: LocationOption) {
    query = option.label;
    showDropdown = false;
    searchStatus = '';
    dispatch('select', option);
  }

  async function handleUseGPS() {
    isLocating = true;
    locationStatus = 'Locating...';
    try {
      const coords = await getCurrentLocation({ timeout: 15000, maximumAge: 0 });
      locationStatus = 'Looking up address...';
      const address = await reverseGeocode(coords.lat, coords.lng);
      
      if (!address || !address.city || !address.country) {
        locationStatus = 'Unable to find address.';
        return;
      }

      const regionLabel = address.region ? `${address.city}, ${address.region}` : address.city;
      const fullLabel = address.region ? `${address.city}, ${address.region}, ${address.country}` : `${address.city}, ${address.country}`;

      const option = {
        label: fullLabel,
        city: regionLabel,
        country: address.country,
        lat: coords.lat,
        lng: coords.lng
      };

      query = fullLabel;
      locationStatus = 'Found!';
      dispatch('select', option);
    } catch (e) {
      locationStatus = 'Location failed.';
      console.error(e);
    } finally {
      isLocating = false;
    }
  }
</script>

<div class="relative">
  <label class="block text-sm font-medium text-slate-700 mb-1" for="location-search">
    {label} {required ? '*' : ''}
  </label>
  <div class="relative">
    <input
      id="location-search"
      type="text"
      bind:value={query}
      {placeholder}
      on:input={handleInput}
      on:focus={handleFocus}
      on:blur={handleBlur}
      class="block w-full rounded-md border-slate-300 shadow-sm focus:border-primary-500 focus:ring-primary-500 sm:text-sm"
    />
    {#if showDropdown && locationOptions.length}
      <div class="absolute z-50 mt-1 w-full bg-white border border-slate-200 rounded-md shadow-lg max-h-56 overflow-auto">
        {#each locationOptions as option}
          <button
            type="button"
            class="w-full text-left px-3 py-2 text-sm text-slate-700 hover:bg-slate-50"
            on:mousedown={() => handleSelect(option)}
          >
            {option.label}
          </button>
        {/each}
      </div>
    {/if}
  </div>
  
  {#if searchStatus}
    <p class="mt-1 text-xs text-slate-500">{searchStatus}</p>
  {/if}

  <div class="mt-2 flex items-center gap-2">
    <button
      type="button"
      on:click={handleUseGPS}
      disabled={isLocating}
      class="text-xs text-primary-600 hover:text-primary-700 font-medium flex items-center gap-1"
    >
      <span>📍</span> Use my current location
    </button>
    {#if locationStatus}
      <span class="text-xs text-slate-500">- {locationStatus}</span>
    {/if}
  </div>
</div>
