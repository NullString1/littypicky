<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { api, type UpdateUserRequest } from '$lib/api';
  import { auth } from '$lib/stores/auth';

  let fullName = '';
  let city = '';
  let country = '';
  let searchRadiusKm = 10;
  
  let loading = false;
  let error = '';
  let success = false;

  onMount(() => {
    // Pre-fill form with current user data
    if ($auth.user) {
      fullName = $auth.user.full_name;
      city = $auth.user.city;
      country = $auth.user.country;
      searchRadiusKm = $auth.user.search_radius_km;
    }
  });

  async function handleSubmit() {
    if (!$auth.token) return;

    try {
      loading = true;
      error = '';
      success = false;

      // Validate
      if (!fullName.trim()) {
        error = 'Full name is required';
        return;
      }
      if (!city.trim()) {
        error = 'City is required';
        return;
      }
      if (!country.trim()) {
        error = 'Country is required';
        return;
      }
      if (searchRadiusKm < 1 || searchRadiusKm > 100) {
        error = 'Search radius must be between 1 and 100 km';
        return;
      }

      // Build update request (only include changed fields)
      const updateData: UpdateUserRequest = {};
      
      if ($auth.user) {
        if (fullName !== $auth.user.full_name) {
          updateData.full_name = fullName;
        }
        if (city !== $auth.user.city) {
          updateData.city = city;
        }
        if (country !== $auth.user.country) {
          updateData.country = country;
        }
        if (searchRadiusKm !== $auth.user.search_radius_km) {
          updateData.search_radius_km = searchRadiusKm;
        }
      }

      // If nothing changed, just go back
      if (Object.keys(updateData).length === 0) {
        goto('/profile/me');
        return;
      }

      const updatedUser = await api.users.updateMe(updateData, $auth.token);
      
      // Update auth store with new user data
      if ($auth.refreshToken) {
        auth.login($auth.token, updatedUser, $auth.refreshToken);
      }

      success = true;
      
      // Redirect back to profile after a short delay
      setTimeout(() => {
        goto('/profile/me');
      }, 1500);

    } catch (e: any) {
      error = e.message || 'Failed to update profile';
    } finally {
      loading = false;
    }
  }

  function handleCancel() {
    goto('/profile/me');
  }
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
    
    <!-- Header -->
    <div class="mb-6">
      <button 
        onclick={handleCancel}
        class="text-primary-600 hover:text-primary-700 text-sm font-medium mb-4 inline-flex items-center"
      >
        ← Back to Profile
      </button>
      <h1 class="text-3xl font-bold text-slate-900">Edit Profile</h1>
      <p class="mt-2 text-sm text-slate-600">Update your personal information and preferences.</p>
    </div>

    {#if error}
      <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6">
        {error}
      </div>
    {/if}

    {#if success}
      <div class="bg-green-50 border border-green-200 text-green-600 px-4 py-3 rounded-md mb-6">
        Profile updated successfully! Redirecting...
      </div>
    {/if}

    <!-- Form -->
    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="bg-white rounded-lg shadow border border-slate-200">
      <div class="px-4 py-5 sm:p-6 space-y-6">
        
        <!-- Full Name -->
        <div>
          <label for="fullName" class="block text-sm font-medium text-slate-700 mb-2">
            Full Name *
          </label>
          <input
            type="text"
            id="fullName"
            bind:value={fullName}
            required
            class="block w-full rounded-md border-slate-300 shadow-sm focus:border-primary-500 focus:ring-primary-500 sm:text-sm"
            placeholder="John Doe"
          />
        </div>

        <!-- Location -->
        <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
          <div>
            <label for="city" class="block text-sm font-medium text-slate-700 mb-2">
              City *
            </label>
            <input
              type="text"
              id="city"
              bind:value={city}
              required
              class="block w-full rounded-md border-slate-300 shadow-sm focus:border-primary-500 focus:ring-primary-500 sm:text-sm"
              placeholder="London"
            />
          </div>

          <div>
            <label for="country" class="block text-sm font-medium text-slate-700 mb-2">
              Country *
            </label>
            <input
              type="text"
              id="country"
              bind:value={country}
              required
              class="block w-full rounded-md border-slate-300 shadow-sm focus:border-primary-500 focus:ring-primary-500 sm:text-sm"
              placeholder="UK"
            />
          </div>
        </div>

        <!-- Search Radius -->
        <div>
          <label for="searchRadius" class="block text-sm font-medium text-slate-700 mb-2">
            Search Radius (km) *
          </label>
          <div class="flex items-center gap-4">
            <input
              type="range"
              id="searchRadius"
              bind:value={searchRadiusKm}
              min="1"
              max="100"
              class="flex-1 h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-600"
            />
            <span class="text-sm font-medium text-slate-900 w-12 text-right">{searchRadiusKm} km</span>
          </div>
          <p class="mt-2 text-sm text-slate-500">
            This determines how far you'll see litter reports from your location.
          </p>
        </div>

        <!-- Email (Read-only) -->
        <div>
          <label for="email" class="block text-sm font-medium text-slate-700 mb-2">
            Email
          </label>
          <input
            type="email"
            id="email"
            value={$auth.user?.email || ''}
            disabled
            class="block w-full rounded-md border-slate-300 bg-slate-50 shadow-sm sm:text-sm cursor-not-allowed"
          />
          <p class="mt-2 text-sm text-slate-500">
            Email cannot be changed. Contact support if you need to update your email.
          </p>
        </div>

      </div>

      <!-- Form Actions -->
      <div class="bg-slate-50 px-4 py-3 sm:px-6 flex items-center justify-end gap-3 rounded-b-lg">
        <button
          type="button"
          onclick={handleCancel}
          class="px-4 py-2 border border-slate-300 rounded-md shadow-sm text-sm font-medium text-slate-700 bg-white hover:bg-slate-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={loading}
          class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading ? 'Saving...' : 'Save Changes'}
        </button>
      </div>
    </form>

    <!-- Additional Information -->
    <div class="mt-6 bg-blue-50 border border-blue-200 rounded-md p-4">
      <div class="flex">
        <div class="flex-shrink-0">
          <span class="text-2xl">ℹ️</span>
        </div>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-blue-800">About your data</h3>
          <div class="mt-2 text-sm text-blue-700">
            <p>Your location information helps us show you relevant litter reports nearby. We never share your personal information with third parties.</p>
          </div>
        </div>
      </div>
    </div>

  </div>
</div>
