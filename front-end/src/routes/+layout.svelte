<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth';

  let { children } = $props();

  onMount(() => {
    auth.initialize();
  });
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
</svelte:head>

<div class="min-h-screen flex flex-col">
  <header class="bg-white border-b border-slate-200 sticky top-0 z-50">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
      <!-- Logo -->
      <a href="/" class="flex items-center gap-2 group">
        <div class="w-8 h-8 bg-primary-500 rounded-lg flex items-center justify-center text-white font-bold text-lg shadow-sm group-hover:bg-primary-600 transition-colors">
          L
        </div>
        <span class="font-bold text-xl tracking-tight text-slate-900">LittyPicky</span>
      </a>

      <!-- Desktop Nav -->
      <nav class="hidden md:flex items-center gap-8">
        <a href="/app/feed" class="text-sm font-medium text-slate-600 hover:text-primary-600 transition-colors">Feed</a>
        <a href="/app/verify" class="text-sm font-medium text-slate-600 hover:text-primary-600 transition-colors">Verify</a>
        <a href="/community" class="text-sm font-medium text-slate-600 hover:text-primary-600 transition-colors">Community</a>
      </nav>

      <!-- Actions -->
      <div class="flex items-center gap-4">
        {#if $auth.isAuthenticated}
            <a href="/app/report" class="bg-primary-600 hover:bg-primary-700 text-white text-sm font-semibold px-4 py-2 rounded-full transition-colors shadow-sm flex items-center gap-2">
              <span>Report Litter</span>
            </a>
            <a href="/profile/me" class="w-9 h-9 bg-slate-100 rounded-full flex items-center justify-center text-slate-600 hover:bg-slate-200 transition-colors">
                <span class="text-xs font-bold">ME</span>
            </a>
        {:else}
            <a href="/auth/login" class="text-sm font-medium text-slate-600 hover:text-slate-900 hidden sm:block">Log in</a>
            <a href="/auth/register" class="bg-primary-600 hover:bg-primary-700 text-white text-sm font-semibold px-4 py-2 rounded-full transition-colors shadow-sm">
              Sign up
            </a>
        {/if}
      </div>
    </div>
  </header>

  <main class="flex-grow">
    {@render children()}
  </main>

  <footer class="bg-white border-t border-slate-200 py-12 mt-auto">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row justify-between items-center gap-6">
      <div class="flex items-center gap-2">
        <div class="w-6 h-6 bg-slate-200 rounded flex items-center justify-center text-slate-500 text-xs font-bold">L</div>
        <span class="text-slate-500 font-semibold text-sm">LittyPicky © 2026</span>
      </div>
      <div class="flex gap-6 text-sm text-slate-500">
        <a href="#" class="hover:text-slate-900">Privacy</a>
        <a href="#" class="hover:text-slate-900">Terms</a>
        <a href="#" class="hover:text-slate-900">About</a>
      </div>
    </div>
  </footer>
</div>
