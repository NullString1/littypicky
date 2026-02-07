<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  let { children } = $props();
  let checked = false;

  onMount(() => {
    // If not authenticated, redirect to login
    // We check localStorage directly as a fallback or trust the store
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
</script>

{#if $auth.isAuthenticated}
    {@render children()}
{:else}
    <!-- Optional: loading state while checking or redirecting -->
    <div class="min-h-[60vh] flex items-center justify-center">
        <div class="w-8 h-8 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
    </div>
{/if}
