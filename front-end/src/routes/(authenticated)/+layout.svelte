<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { page } from '$app/stores';

  let { children } = $props();
  let checked = $state(false);
  let isInitializing = $state(true);

  // Initialize auth immediately if in browser
  if (browser) {
    auth.initialize();
    isInitializing = false;
  }

  onMount(() => {
    // If not authenticated, redirect to login
    if (browser) {
        if (!$auth.isAuthenticated) {
             const token = localStorage.getItem('token');
             if (!token) {
                 goto(`/auth/login?redirect=${window.location.pathname}`);
             }
        }
        checked = true;
    }
  });

  // Reactive check for logout
  $effect(() => {
      if (browser && checked && !$auth.isAuthenticated) {
          goto('/auth/login');
      }
  });

  $effect(() => {
      if (!browser || !checked || !$auth.isAuthenticated || !$auth.user) {
          return;
      }
      const path = $page.url.pathname;
      const isProfileEdit = path === '/profile/me/edit';
      const hasUnknownLocation =
          $auth.user.city === 'Unknown' || $auth.user.country === 'Unknown';
      if (hasUnknownLocation && !isProfileEdit) {
          goto('/profile/me/edit');
      }
  });
</script>

{#if isInitializing}
    <div class="min-h-[60vh] flex items-center justify-center">
        <div class="w-8 h-8 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
    </div>
{:else if $auth.isAuthenticated}
    {@render children()}
{:else}
    <!-- Loading state while checking or redirecting -->
    <div class="min-h-[60vh] flex items-center justify-center">
        <div class="w-8 h-8 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
    </div>
{/if}
