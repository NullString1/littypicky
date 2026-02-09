<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { goto, replaceState } from '$app/navigation';
  import { browser } from '$app/environment';
  import { getCurrentLocation, reverseGeocode } from '$lib/utils/geolocation';


  let isLoading = false;
  let error = '';
  let isLocating = false;
  let locationStatus = '';
  type LocationOption = {
    label: string;
    city: string;
    country: string;
  };

  let locationOptions: LocationOption[] = [];
  let citySearchTimeout: ReturnType<typeof setTimeout> | null = null;
  let citySearchStatus = '';
  let showCityDropdown = false;
  let hideCityDropdownTimeout: ReturnType<typeof setTimeout> | null = null;
  let city = '';
  let country = '';
  let locationQuery = '';
  let locationPlaceholder = 'Start typing your city';
  let hasSelectedLocation = false;

  onMount(() => {
    if (browser && $auth.isAuthenticated) {
      goto('/app/feed');
    }
    locationPlaceholder = 'Start typing your city';
  });

  async function handleSubmit(event: Event) {
    event.preventDefault();
    isLoading = true;
    error = '';
    
    const form = event.target as HTMLFormElement;
    const formData = new FormData(form);
    
    const email = formData.get('email') as string;
    const password = formData.get('password') as string;
    const full_name = formData.get('name') as string;
    if (!hasSelectedLocation || !city || !country) {
      error = 'Please select a location from the suggestions.';
      isLoading = false;
      return;
    }

    if (!hasSelectedLocation) {
      const locationIsValid = await validateLocationSelection(city, country);
      if (!locationIsValid) {
        error = 'Please select a location from the suggestions.';
        isLoading = false;
        return;
      }
    }
    
    try {
      await api.auth.register({
        email,
        password,
        full_name,
        city,
        country
      });
      // Redirect to login or show success message
      // The backend says "User registered successfully. Verification email sent."
      goto('/auth/login?fromRegister=true', { replaceState: true, invalidateAll: true});
    } catch (e: any) {
      error = e.message;
    } finally {
      isLoading = false;
    }
  }

  async function handleUseLocation() {
    isLocating = true;
    error = '';
    locationStatus = 'Locating...';

    try {
      const coords = await getCurrentLocation({
        maximumAge: 0,
        timeout: 15000
      });
      locationStatus = 'Looking up your city and country...';

      const address = await reverseGeocode(coords.lat, coords.lng) as {
        city: string;
        region?: string;
        country: string;
      } | null;
      if (!address || !address.city || !address.country) {
        locationStatus = 'Unable to determine city/country. Please select manually.';
        return;
      }

      const regionLabel = address.region ? `${address.city}, ${address.region}` : address.city;
      const fullLabel = address.region ? `${address.city}, ${address.region}, ${address.country}` : `${address.city}, ${address.country}`;
      city = regionLabel;
      country = address.country;
      locationOptions = [{
        label: fullLabel,
        city: regionLabel,
        country: address.country
      }];
      locationQuery = fullLabel;
      locationPlaceholder = fullLabel;
      hasSelectedLocation = true;
      showCityDropdown = false;

      locationStatus = `Found: ${address.city}, ${address.country}`;
    } catch (err: any) {
      locationStatus = 'Location access failed. Please select manually.';
      console.error('Location error:', err);
    } finally {
      isLocating = false;
    }
  }

  function handleCityQueryInput(event: Event) {
    const value = (event.target as HTMLInputElement).value.trim();
    locationQuery = value;
    city = '';
    country = '';
    hasSelectedLocation = false;
    citySearchStatus = '';
    if (!value || value.length < 2) {
      showCityDropdown = false;
      return;
    }

    if (citySearchTimeout) {
      clearTimeout(citySearchTimeout);
    }

    citySearchTimeout = setTimeout(() => {
      void loadLocationOptions(value);
    }, 300);
  }

  function handleCityFocus() {
    if (locationOptions.length) {
      showCityDropdown = true;
    }
  }

  function handleCityBlur() {
    if (hideCityDropdownTimeout) {
      clearTimeout(hideCityDropdownTimeout);
    }
    hideCityDropdownTimeout = setTimeout(() => {
      showCityDropdown = false;
    }, 150);
  }

  function handleCitySelect(option: LocationOption) {
    city = option.city;
    country = option.country;
    locationQuery = option.label;
    hasSelectedLocation = true;
    showCityDropdown = false;
    citySearchStatus = '';
  }

  async function loadLocationOptions(query: string): Promise<LocationOption[]> {
    try {
      citySearchStatus = 'Searching...';
      const response = await fetch(
        `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(query)}&count=10&language=en&format=json`
      );
      const data = await response.json();
      const results = Array.isArray(data?.results) ? data.results : [];
      const mapped = results
        .map((item: any) => {
          const name = item?.name as string | undefined;
          if (!name) return null;
          const admin = item?.admin1 as string | undefined;
          const admin2 = item?.admin2 as string | undefined;
          const countryName = item?.country as string | undefined;
          const labelParts = [name, admin, admin2, countryName].filter(Boolean);
          return {
            label: labelParts.join(', '),
            city: admin ? `${name}, ${admin}` : name,
            country: countryName || ''
          };
        })
        .filter((item: LocationOption | null): item is LocationOption => Boolean(item));

      const uniqueMap = new Map<string, LocationOption>();
      mapped.forEach((item: LocationOption) => {
        if (!uniqueMap.has(item.label)) {
          uniqueMap.set(item.label, item);
        }
      });

      const unique = Array.from(uniqueMap.values()).slice(0, 10);
      locationOptions = unique;
      citySearchStatus = locationOptions.length ? '' : 'No matches found.';
      showCityDropdown = locationOptions.length > 0;
      return locationOptions;
    } catch (err) {
      console.error('City lookup failed:', err);
      locationOptions = [];
      citySearchStatus = 'No matches found.';
      showCityDropdown = false;
      return [];
    }
  }

  async function validateLocationSelection(candidateCity: string, candidateCountry: string): Promise<boolean> {
    if (!candidateCity || !candidateCountry) {
      return false;
    }

    if (locationOptions.some((option) => option.city === candidateCity && option.country === candidateCountry)) {
      return true;
    }

    const options = await loadLocationOptions(locationQuery || candidateCity);
    return options.some((option) => option.city === candidateCity && option.country === candidateCountry);
  }

  function availableCityOptions(): LocationOption[] {
    return locationOptions;
  }
</script>

<div class="min-h-[calc(100vh-4rem)] flex flex-col justify-center py-12 sm:px-6 lg:px-8 bg-slate-50">
  <div class="sm:mx-auto sm:w-full sm:max-w-md">
    <div class="flex justify-center">
       <div class="w-12 h-12 bg-primary-500 rounded-xl flex items-center justify-center text-white font-bold text-2xl shadow-sm">L</div>
    </div>
    <h2 class="mt-6 text-center text-3xl font-extrabold text-slate-900">
      Join the movement
    </h2>
    <p class="mt-2 text-center text-sm text-slate-600">
      Already have an account?
      <a href="/auth/login" class="font-medium text-primary-600 hover:text-primary-500">
        Sign in
      </a>
    </p>
  </div>

  <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
    <div class="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10 border border-slate-200">
      {#if error}
        <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6 text-sm">
          {error}
        </div>
      {/if}

      <form class="space-y-6" onsubmit={handleSubmit}>
        <div>
            <label for="name" class="block text-sm font-medium text-slate-700"> Full Name </label>
            <div class="mt-1">
              <input id="name" name="name" type="text" autocomplete="name" required class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
            </div>
        </div>

        <div>
          <label for="email" class="block text-sm font-medium text-slate-700"> Email address </label>
          <div class="mt-1">
            <input id="email" name="email" type="email" autocomplete="email" required class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
          </div>
        </div>

        <div>
          <label for="city" class="block text-sm font-medium text-slate-700"> Location </label>
          <div class="mt-1 relative">
            <input
              id="city"
              name="city"
              type="text"
              autocomplete="off"
              placeholder={locationPlaceholder}
              required
              bind:value={locationQuery}
              oninput={handleCityQueryInput}
              onfocus={handleCityFocus}
              onblur={handleCityBlur}
              class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm"
            >
            {#if showCityDropdown && availableCityOptions().length}
              <div class="absolute z-10 mt-1 w-full bg-white border border-slate-200 rounded-md shadow-lg max-h-56 overflow-auto">
                {#each availableCityOptions() as option}
                  <button
                    type="button"
                    class="w-full text-left px-3 py-2 text-sm text-slate-700 hover:bg-slate-50"
                    onmousedown={() => handleCitySelect(option)}
                  >
                    {option.label}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
          {#if citySearchStatus}
            <p class="mt-2 text-xs text-slate-500">{citySearchStatus}</p>
          {/if}
        </div>

        <div>
          <button type="button" onclick={handleUseLocation} disabled={isLocating} class="inline-flex items-center px-3 py-2 border border-slate-300 shadow-sm text-sm leading-4 font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50 disabled:opacity-50">
            Use GPS to fill city/country
          </button>
          {#if locationStatus}
            <p class="mt-2 text-xs text-slate-500">{locationStatus}</p>
          {/if}
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-slate-700"> Password </label>
          <div class="mt-1">
            <input id="password" name="password" type="password" autocomplete="new-password" required class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
          </div>
        </div>

        <div>
          <button type="submit" disabled={isLoading} class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed">
            {isLoading ? 'Creating Account...' : 'Create Account'}
          </button>
        </div>
      </form>

      <div class="mt-6">
        <div class="relative">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-slate-300"></div>
          </div>
          <div class="relative flex justify-center text-sm">
            <span class="px-2 bg-white text-slate-500"> Or sign up with </span>
          </div>
        </div>

        <div class="mt-6">
          <a href="/api/auth/google" class="w-full inline-flex justify-center py-2 px-4 border border-slate-300 rounded-md shadow-sm bg-white text-sm font-medium text-slate-500 hover:bg-slate-50">
            <span class="sr-only">Sign up with Google</span>
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
               <path d="M12.48 10.92v3.28h7.84c-.24 1.84-.853 3.187-1.787 4.133-1.147 1.147-2.933 2.4-6.053 2.4-4.827 0-8.6-3.893-8.6-8.72s3.773-8.72 8.6-8.72c2.6 0 4.507 1.027 5.907 2.347l2.307-2.307C18.747 1.44 16.133 0 12.48 0 5.867 0 .307 5.387.307 12s5.56 12 12.173 12c3.573 0 6.267-1.173 8.373-3.36 2.16-2.16 2.84-5.213 2.84-7.667 0-.76-.053-1.467-.173-2.053H12.48z"/>
            </svg>
            <span class="ml-2">Google</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
