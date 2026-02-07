<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let status: 'loading' | 'success' | 'error' = 'loading';
  let message = 'Verifying your email...';

  onMount(async () => {
    const token = $page.url.searchParams.get('token');
    
    if (!token) {
      status = 'error';
      message = 'No verification token found.';
      return;
    }

    try {
      const tokens = await api.auth.verifyEmail(token);
      // Auto-login after verification
      auth.login(tokens.access_token, tokens.user);
      status = 'success';
      message = 'Email verified successfully! Redirecting...';
      setTimeout(() => {
          goto('/app/feed');
      }, 2000);
    } catch (e: any) {
      status = 'error';
      message = e.message || 'Verification failed. The token may be invalid or expired.';
    }
  });
</script>

<div class="min-h-[calc(100vh-4rem)] flex flex-col justify-center items-center py-12 sm:px-6 lg:px-8 bg-slate-50">
  <div class="max-w-md w-full bg-white shadow rounded-lg p-8 text-center">
    {#if status === 'loading'}
      <div class="w-16 h-16 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin mx-auto mb-4"></div>
      <h2 class="text-2xl font-bold text-slate-900 mb-2">Verifying...</h2>
      <p class="text-slate-600">{message}</p>
    {:else if status === 'success'}
        <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
            </svg>
        </div>
        <h2 class="text-2xl font-bold text-slate-900 mb-2">Verified!</h2>
        <p class="text-slate-600 mb-6">{message}</p>
        <a href="/app/feed" class="inline-flex justify-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700">
            Go to Feed
        </a>
    {:else}
        <div class="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
        </div>
        <h2 class="text-2xl font-bold text-slate-900 mb-2">Verification Failed</h2>
        <p class="text-red-600 mb-6">{message}</p>
        <div class="flex flex-col gap-3">
            <a href="/auth/login" class="text-primary-600 hover:text-primary-500 font-medium">
                Back to Login
            </a>
            <button class="text-slate-500 hover:text-slate-700 text-sm">
                Resend verification email
            </button>
        </div>
    {/if}
  </div>
</div>
