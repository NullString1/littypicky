<script lang="ts">
  import { api } from '$lib/api';
  import { page } from '$app/state';

  let newPassword = $state('');
  let confirmPassword = $state('');
  let status = $state<'idle' | 'submitting' | 'success' | 'error'>('idle');
  let message = $state('');

  let token = $derived(page.url.searchParams.get('token') ?? '');

  function validatePassword(value: string) {
    if (value.length < 8) return 'Password must be at least 8 characters.';
    return '';
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();

    if (!token) {
      status = 'error';
      message = 'Invalid or missing reset token.';
      return;
    }

    const validationError = validatePassword(newPassword);
    if (validationError) {
      status = 'error';
      message = validationError;
      return;
    }

    if (newPassword !== confirmPassword) {
      status = 'error';
      message = 'Passwords do not match.';
      return;
    }

    status = 'submitting';
    message = '';

    try {
      const result = await api.auth.resetPassword({ token, new_password: newPassword });
      status = 'success';
      message = result.message;
      newPassword = '';
      confirmPassword = '';
    } catch (e: any) {
      status = 'error';
      message = e.message || 'Unable to reset password.';
    }
  }
</script>

<div class="min-h-[calc(100vh-4rem)] flex flex-col justify-center py-12 sm:px-6 lg:px-8 bg-slate-50">
  <div class="sm:mx-auto sm:w-full sm:max-w-md">
    <div class="flex justify-center">
      <div class="w-12 h-12 bg-primary-500 rounded-xl flex items-center justify-center text-white font-bold text-2xl shadow-sm">L</div>
    </div>
    <h2 class="mt-6 text-center text-3xl font-extrabold text-slate-900">Reset your password</h2>
    <p class="mt-2 text-center text-sm text-slate-600">Enter a new password for your account.</p>
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
          <label for="new-password" class="block text-sm font-medium text-slate-700"> New password </label>
          <div class="mt-1">
            <input id="new-password" name="new-password" type="password" autocomplete="new-password" required bind:value={newPassword} class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
          </div>
        </div>

        <div>
          <label for="confirm-password" class="block text-sm font-medium text-slate-700"> Confirm password </label>
          <div class="mt-1">
            <input id="confirm-password" name="confirm-password" type="password" autocomplete="new-password" required bind:value={confirmPassword} class="appearance-none block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm placeholder-slate-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm">
          </div>
        </div>

        <div>
          <button type="submit" disabled={status === 'submitting'} class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed">
            {status === 'submitting' ? 'Resetting...' : 'Reset password'}
          </button>
        </div>
      </form>

      <div class="mt-6 text-center text-sm">
        <a href="/auth/login" class="font-medium text-primary-600 hover:text-primary-500">Back to sign in</a>
      </div>
    </div>
  </div>
</div>
