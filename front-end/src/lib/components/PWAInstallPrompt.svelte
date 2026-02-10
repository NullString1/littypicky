<script lang="ts">
  import { onMount } from 'svelte';

  let deferredPrompt: any = null;
  let showInstallPrompt = false;

  onMount(() => {
    window.addEventListener('beforeinstallprompt', (e) => {
      // Prevent the mini-infobar from appearing on mobile
      e.preventDefault();
      // Stash the event so it can be triggered later.
      deferredPrompt = e;
      // Update UI notify the user they can install the PWA
      showInstallPrompt = true;
    });

    window.addEventListener('appinstalled', () => {
      // Hide the app-provided install promotion
      showInstallPrompt = false;
      // Clear the deferred prompt so it can be garbage collected
      deferredPrompt = null;
    });
  });

  async function installPWA() {
    if (!deferredPrompt) return;
    // Show the install prompt
    deferredPrompt.prompt();
    // Wait for the user to respond to the prompt
    const { outcome } = await deferredPrompt.userChoice;
    // We've used the prompt, and can't use it again, throw it away
    deferredPrompt = null;
    showInstallPrompt = false;
  }

  function dismiss() {
    showInstallPrompt = false;
  }
</script>

{#if showInstallPrompt}
  <div class="fixed bottom-4 left-4 right-4 md:left-auto md:right-4 md:w-96 bg-white border border-slate-200 rounded-lg shadow-lg p-4 z-50 animate-fade-in-up">
    <div class="flex items-start justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 bg-primary-100 rounded-lg flex items-center justify-center text-primary-600">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect width="16" height="16" x="4" y="4" rx="2"/><path d="M12 9v6"/><path d="M9 12h6"/>
          </svg>
        </div>
        <div>
          <h3 class="font-semibold text-slate-900">Install App</h3>
          <p class="text-sm text-slate-500">Add LittyPicky to your home screen for better experience.</p>
        </div>
      </div>
      <button onclick={dismiss} class="text-slate-400 hover:text-slate-600" aria-label="Dismiss">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 6 6 18"/><path d="m6 6 12 12"/>
        </svg>
      </button>
    </div>
    <div class="mt-4 flex gap-2 justify-end">
      <button onclick={dismiss} class="px-3 py-1.5 text-sm font-medium text-slate-600 hover:text-slate-800 transition-colors">
        Later
      </button>
      <button onclick={installPWA} class="px-3 py-1.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors shadow-sm">
        Install
      </button>
    </div>
  </div>
{/if}

<style>
  @keyframes fade-in-up {
    from {
      opacity: 0;
      transform: translateY(1rem);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .animate-fade-in-up {
    animation: fade-in-up 0.3s ease-out forwards;
  }
</style>
