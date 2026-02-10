<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, type Report, type CreateVerificationRequest, type ClearReportRequest } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import imageCompression from 'browser-image-compression';
  import { getStatusColor } from '$lib/utils/status';
  import { formatDateTime } from '$lib/utils/date';

  let report: Report | undefined = $state();
  let loading = $state(true);
  let error = $state('');
  let claiming = $state(false);
  let clearing = $state(false);
  let verifying = $state(false);
  
  let clearPhotoPreview = $state<string | null>(null);
  let clearPhotoBase64 = $state<string | null>(null);
  
  let isLegitimate = $state(true);
  let verifyComments = $state('');

  let reportId = $derived(page.params.id);

  onMount(async () => {
    await loadReport();
  });

  async function loadReport() {
    if (!$auth.token) return;
    if (!reportId) {
      await goto('/app/feed');
      return;
    }
    try {
      loading = true;
      error = '';
      report = await api.reports.getById(reportId, $auth.token);
      if (!report) {
        await goto('/app/feed');
      }
    } catch (e: any) {
      error = e.message || 'Failed to load report';
      await goto('/app/feed');
    } finally {
      loading = false;
    }
  }

  function handleClearFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      const file = input.files[0];
      
      if (file.size > 5 * 1024 * 1024) {
        error = 'File size must be less than 5MB';
        return;
      }

      const reader = new FileReader();
      reader.onload = (e) => {
        clearPhotoPreview = e.target?.result as string;
        clearPhotoBase64 = clearPhotoPreview;
      };
      reader.readAsDataURL(file);
    }
  }

  async function claimReport() {
    if (!$auth.token || !report) return;
    
    try {
      claiming = true;
      error = '';
      await api.reports.claim(report.id, $auth.token);
      await loadReport(); // Reload to get updated status
    } catch (e: any) {
      error = e.message || 'Failed to claim report';
    } finally {
      claiming = false;
    }
  }

  async function clearReport() {
    if (!$auth.token || !report || !clearPhotoBase64) {
      error = 'Please upload an after photo';
      return;
    }
    
    try {
      clearing = true;
      error = '';
      const data: ClearReportRequest = {
        photo_base64: clearPhotoBase64
      };
      await api.reports.clear(report.id, data, $auth.token);
      await loadReport(); // Reload to get updated status
      clearPhotoPreview = null;
      clearPhotoBase64 = null;
    } catch (e: any) {
      error = e.message || 'Failed to clear report';
    } finally {
      clearing = false;
    }
  }

  async function verifyReport() {
    if (!$auth.token || !report) {
      error = 'Unable to verify report';
      return;
    }
    
    try {
      verifying = true;
      error = '';
      const data: CreateVerificationRequest = {
        is_verified: isLegitimate,
        comment: verifyComments || undefined
      };
      await api.reports.verify(report.id, data, $auth.token);
      await loadReport(); // Reload to get updated status
      verifyComments = '';
    } catch (e: any) {
      error = e.message || 'Failed to verify report';
    } finally {
      verifying = false;
    }
  }

  function openDirections(lat: number, lng: number) {
    window.open(`https://www.google.com/maps/dir/?api=1&destination=${lat},${lng}`, '_blank');
  }
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
    
    {#if loading}
      <div class="flex justify-center py-12">
        <div class="w-12 h-12 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
      </div>
    {:else if error && !report}
      <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6">
        {error}
      </div>
      <a href="/app/feed" class="text-primary-600 hover:text-primary-700">← Back to Feed</a>
    {:else if report}
      <!-- Header -->
      <div class="mb-6">
        <a href="/app/feed" class="text-primary-600 hover:text-primary-700 text-sm font-medium mb-4 inline-block">← Back to Feed</a>
        <div class="flex items-center justify-between">
          <h1 class="text-3xl font-bold text-slate-900">{report.address || `${report.latitude.toFixed(4)}, ${report.longitude.toFixed(4)}`}</h1>
          <span class={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(report.status)} uppercase tracking-wide`}>
            {report.status}
          </span>
        </div>
        <p class="text-slate-500 mt-2">Reported on {formatDateTime(report.created_at)}</p>
      </div>

      {#if error}
        <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6">
          {error}
        </div>
      {/if}

      <!-- Images -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
        <!-- Before Photo -->
        <div class="bg-white rounded-lg shadow border border-slate-200 overflow-hidden">
          <div class="px-4 py-3 border-b border-slate-200">
            <h3 class="text-sm font-semibold text-slate-700">Before Photo</h3>
          </div>
          {#if report.photo_before}
            <img 
              src={report.photo_before} 
              alt="Before cleanup" 
              class="w-full h-64 object-cover"
              loading="lazy"
              decoding="async"
            />
          {:else}
            <div class="w-full h-64 bg-slate-100 flex items-center justify-center text-slate-400">
              <span class="text-4xl">📸</span>
            </div>
          {/if}
        </div>

        <!-- After Photo -->
        {#if report.photo_after}
          <div class="bg-white rounded-lg shadow border border-slate-200 overflow-hidden">
            <div class="px-4 py-3 border-b border-slate-200">
              <h3 class="text-sm font-semibold text-slate-700">After Photo</h3>
            </div>
            <img 
              src={report.photo_after} 
              alt="After cleanup" 
              class="w-full h-64 object-cover"
              loading="lazy"
              decoding="async"
            />
          </div>
        {/if}
      </div>

      <!-- Details -->
      <div class="bg-white rounded-lg shadow border border-slate-200 p-6 mb-6">
        <h2 class="text-xl font-bold text-slate-900 mb-4">Report Details</h2>
        
        <dl class="grid grid-cols-1 gap-4">
          <div>
            <dt class="text-sm font-medium text-slate-500">Description</dt>
            <dd class="mt-1 text-sm text-slate-900">{report.description || 'No description provided'}</dd>
          </div>
          
          <div class="grid grid-cols-2 gap-4">
            <div>
              <dt class="text-sm font-medium text-slate-500">Location</dt>
              <dd class="mt-1 text-sm text-slate-900">{report.address || 'Unknown address'}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-slate-500">Coordinates</dt>
              <dd class="mt-1 text-sm text-slate-900">{report.latitude.toFixed(4)}, {report.longitude.toFixed(4)}</dd>
            </div>
          </div>

          {#if report.claimed_at}
            <div>
              <dt class="text-sm font-medium text-slate-500">Claimed At</dt>
              <dd class="mt-1 text-sm text-slate-900">{formatDateTime(report.claimed_at)}</dd>
            </div>
          {/if}

          {#if report.cleared_at}
            <div>
              <dt class="text-sm font-medium text-slate-500">Cleared At</dt>
              <dd class="mt-1 text-sm text-slate-900">{formatDateTime(report.cleared_at)}</dd>
            </div>
          {/if}
        </dl>

        <div class="mt-6">
          <button 
            onclick={() => report && openDirections(report.latitude, report.longitude)}
            class="w-full sm:w-auto inline-flex items-center justify-center px-4 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50"
          >
            📍 Get Directions
          </button>
        </div>
      </div>

      <!-- Actions based on status -->
      {#if report.status === 'pending' && report.reporter_id !== $auth.user?.id}
        <div class="bg-white rounded-lg shadow border border-slate-200 p-6">
          <h2 class="text-xl font-bold text-slate-900 mb-4">Claim This Report</h2>
          <p class="text-slate-600 mb-4">Be the first to clean up this area! Claim this report to let others know you're on it.</p>
          <button 
            onclick={claimReport}
            disabled={claiming}
            class="w-full sm:w-auto inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {claiming ? 'Claiming...' : 'Claim Report'}
          </button>
        </div>
      {/if}

      {#if report.status === 'claimed' && report.claimed_by === $auth.user?.id}
        <div class="bg-white rounded-lg shadow border border-slate-200 p-6">
          <h2 class="text-xl font-bold text-slate-900 mb-4">Mark as Cleared</h2>
          <p class="text-slate-600 mb-4">Upload an after photo showing the cleaned area.</p>
          
          <div class="mb-4">
            <label class="block text-sm font-medium text-slate-700 mb-2">
              After Photo *
              <input 
                type="file" 
                accept="image/*" 
                onchange={handleClearFileSelect}
                class="block w-full text-sm text-slate-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-primary-50 file:text-primary-700 hover:file:bg-primary-100"
              />
            </label>
          </div>

          {#if clearPhotoPreview}
            <div class="mb-4">
              <img 
                src={clearPhotoPreview} 
                alt="Preview" 
                class="max-w-xs rounded-lg border border-slate-200"
                loading="lazy"
                decoding="async"
              />
            </div>
          {/if}

          <button 
            onclick={clearReport}
            disabled={clearing || !clearPhotoBase64}
            class="w-full sm:w-auto inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {clearing ? 'Submitting...' : 'Submit Cleared Report'}
          </button>
        </div>
      {/if}

      {#if report.status === 'cleared'}
        <div class="bg-white rounded-lg shadow border border-slate-200 p-6">
          <h2 class="text-xl font-bold text-slate-900 mb-4">Verify This Report</h2>
          <p class="text-slate-600 mb-4">Help the community by verifying this cleanup. Confirm that the area has been properly cleaned.</p>
          
          <div class="mb-4">
            <label class="flex items-center">
              <input 
                type="checkbox" 
                bind:checked={isLegitimate}
                class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-slate-300 rounded"
              />
              <span class="ml-2 text-sm text-slate-700">This cleanup is legitimate</span>
            </label>
          </div>

          <div class="mb-4">
            <label class="block text-sm font-medium text-slate-700 mb-2">
              Comments (optional)
              <textarea 
                bind:value={verifyComments}
                rows="3"
                class="block w-full rounded-md border-slate-300 shadow-sm focus:border-primary-500 focus:ring-primary-500 sm:text-sm"
                placeholder="Add any additional comments..."
              ></textarea>
            </label>
          </div>

          <button 
            onclick={verifyReport}
            disabled={verifying}
            class="w-full sm:w-auto inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {verifying ? 'Submitting...' : 'Submit Verification'}
          </button>
        </div>
      {/if}

      {#if report.status === 'verified'}
        <div class="bg-green-50 border border-green-200 rounded-lg p-6">
          <div class="flex items-center">
            <span class="text-3xl mr-3">✅</span>
            <div>
              <h3 class="text-lg font-semibold text-green-900">Verified Cleanup!</h3>
              <p class="text-green-700 mt-1">This report has been verified by the community. Great work!</p>
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>
