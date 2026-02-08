<script lang="ts">
  import { api } from '$lib/api';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';

  let email = $state('');
  let status = $state<'idle' | 'submitting' | 'success' | 'error'>('idle');
  let message = $state('');

  onMount(() => {
    if (browser && $auth.isAuthenticated) {
      goto('/app/feed');
    }
  });

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (!email) {
      status = 'error';
      message = 'Please enter your email address.';
      return;
    }

    status = 'submitting';
    message = '';

    try {
      const result = await api.auth.forgotPassword({ email });
      status = 'success';
      message = result.message;
      email = '';
    } catch (e: any) {
      status = 'error';
      message = e.message || 'Unable to send reset link.';
    }
  }
</script>

<div class="min-h-[calc(100vh-4rem)] flex flex-col justify-center py-12 sm:px-6 lg:px-8 bg-slate-50">
  <div class="sm:mx-auto sm:w-full sm:max-w-md">
    <div class="flex justify-center">
      <div class="w-12 h-12 bg-primary-500 rounded-xl flex items-center justify-center text-white font-bold text-2xl shadow-sm">L</div>
    </div>
    <h2 class="mt-6 text-center text-3xl font-extrabold text-slate-900">Forgot your password?</h2>
    <p class="mt-2 text-center text-sm text-slate-600">Enter your email and we will send a reset link.</p>
  </div>

  <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
    <div class="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10 border border-slate-200">
      {#if status === 'error'}
        <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6 text-sm">
          {message}
        </div>
      {/if}

      {#if status === 'success'}
        <div class="bg-green-50 border border-green-200 text-green-600 px-4 py-3 rounded-md mb-6 text-sm">
          {message}
        </div>
      {/if}

      <form class="space-y-6" onsubmit={handleSubmit}>
        <div>
          <label for="email" class="block text-sm font-medium text-slate-700"> Email address </label>
          <div class="mt-1">
            <input id="email" name="email" type="email" autocomplete="email" required bind:value={email} class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
          </div>
        </div>

        <div>
          <button type="submit" disabled={status === 'submitting'} class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed">
            {status === 'submitting' ? 'Sending...' : 'Send reset link'}
          </button>
        </div>
      </form>

      <div class="mt-6 text-center text-sm">
        <a href="/auth/login" class="font-medium text-primary-600 hover:text-primary-500">Back to sign in</a>
      </div>
    </div>
  </div>
</div>
