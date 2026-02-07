<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  let isLoading = false;
  let error = '';

  onMount(() => {
    if (browser && $auth.isAuthenticated) {
      goto('/app/feed');
    }
  });

  async function handleSubmit(event: Event) {
    event.preventDefault();
    isLoading = true;
    error = '';
    
    const form = event.target as HTMLFormElement;
    const formData = new FormData(form);
    
    const email = formData.get('email') as string;
    const password = formData.get('password') as string;
    
    try {
      const tokens = await api.auth.login({
        email,
        password
      });
      
      auth.login(tokens.access_token, tokens.user, tokens.refresh_token);
      goto('/app/feed');
    } catch (e: any) {
      error = e.message;
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="min-h-[calc(100vh-4rem)] flex flex-col justify-center py-12 sm:px-6 lg:px-8 bg-slate-50">
  <div class="sm:mx-auto sm:w-full sm:max-w-md">
    <div class="flex justify-center">
       <div class="w-12 h-12 bg-primary-500 rounded-xl flex items-center justify-center text-white font-bold text-2xl shadow-sm">L</div>
    </div>
    <h2 class="mt-6 text-center text-3xl font-extrabold text-slate-900">
      Sign in to your account
    </h2>
    <p class="mt-2 text-center text-sm text-slate-600">
      Or
      <a href="/auth/register" class="font-medium text-primary-600 hover:text-primary-500">
        create a new account
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
          <label for="email" class="block text-sm font-medium text-slate-700"> Email address </label>
          <div class="mt-1">
            <input id="email" name="email" type="email" autocomplete="email" required class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
          </div>
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-slate-700"> Password </label>
          <div class="mt-1">
            <input id="password" name="password" type="password" autocomplete="current-password" required class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
          </div>
        </div>

        <div class="flex items-center justify-between">
          <div class="flex items-center">
            <input id="remember-me" name="remember-me" type="checkbox" class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-slate-300 rounded">
            <label for="remember-me" class="ml-2 block text-sm text-slate-900"> Remember me </label>
          </div>

          <div class="text-sm">
            <a href="#" class="font-medium text-primary-600 hover:text-primary-500"> Forgot your password? </a>
          </div>
        </div>

        <div>
          <button type="submit" disabled={isLoading} class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed">
            {isLoading ? 'Signing in...' : 'Sign in'}
          </button>
        </div>
      </form>

      <div class="mt-6">
        <div class="relative">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-slate-300"></div>
          </div>
          <div class="relative flex justify-center text-sm">
            <span class="px-2 bg-white text-slate-500"> Or continue with </span>
          </div>
        </div>

        <div class="mt-6">
          <a href="#" class="w-full inline-flex justify-center py-2 px-4 border border-slate-300 rounded-md shadow-sm bg-white text-sm font-medium text-slate-500 hover:bg-slate-50">
            <span class="sr-only">Sign in with Google</span>
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
