<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type Report } from '$lib/api';
  import { auth } from '$lib/stores/auth';

  let queue: Report[] = [];
  let loading = true;
  let error = '';
  let userLocation: { lat: number; lng: number } | null = null;

  async function loadQueue() {
    loading = true;
    error = '';
    try {
        if (!$auth.token) return;

        // Default location if geolocation fails
        let lat = 51.5074;
        let lng = -0.1278;

        if (userLocation) {
            lat = userLocation.lat;
            lng = userLocation.lng;
        }

        const data = await api.reports.getVerificationQueue(lat, lng, 50, $auth.token); // 50km radius
        queue = data;
    } catch (e: any) {
        error = e.message || 'Failed to load verification queue';
    } finally {
        loading = false;
    }
  }

  onMount(() => {
    if (navigator.geolocation) {
        navigator.geolocation.getCurrentPosition(
            (position) => {
                userLocation = {
                    lat: position.coords.latitude,
                    lng: position.coords.longitude
                };
                loadQueue();
            },
            (err) => {
                console.warn('Geolocation denied or failed:', err);
                loadQueue();
            }
        );
    } else {
        loadQueue();
    }
  });

  async function handleVerify(id: string, decision: 'accept' | 'deny') {
    if (!$auth.token) return;
    
    // Optimistic update
    queue = queue.filter(r => r.id !== id);

    try {
        await api.reports.verify(id, {
            is_verified: decision === 'accept',
            comment: decision === 'accept' ? 'Verified by community' : 'Rejected by community'
        }, $auth.token);
        
        // alert(`Claim ${decision === 'accept' ? 'ACCEPTED' : 'DENIED'}. Points updated!`);
    } catch (e: any) {
        alert(`Failed to submit verification: ${e.message}`);
        // Reload on error
        loadQueue();
    }
  }
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    
    <div class="md:flex md:items-center md:justify-between mb-8">
      <div class="flex-1 min-w-0">
        <h2 class="text-2xl font-bold leading-7 text-slate-900 sm:text-3xl sm:truncate">
          Verification Queue
        </h2>
        <p class="mt-1 text-sm text-slate-500">
          Review cleanup claims from fellow pickers. Help keep the community honest!
        </p>
      </div>
      <div class="mt-4 flex md:mt-0 md:ml-4">
          <span class="inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium bg-primary-100 text-primary-800">
            {queue.length} Pending
          </span>
      </div>
    </div>

    {#if loading}
        <div class="flex justify-center py-12">
            <div class="w-12 h-12 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
        </div>
    {:else if error}
        <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6">
            {error}
        </div>
    {:else}
        <div class="space-y-8">
        {#each queue as report}
            <div class="bg-white shadow overflow-hidden sm:rounded-lg border border-slate-200">
                <div class="px-4 py-5 border-b border-slate-200 sm:px-6">
                    <h3 class="text-lg leading-6 font-medium text-slate-900">
                        Cleanup at {report.city}
                    </h3>
                    <p class="mt-1 text-sm text-slate-500">
                        Cleared on {report.cleared_at ? new Date(report.cleared_at).toLocaleDateString() : 'Unknown'}
                    </p>
                </div>
                
                <div class="px-4 py-5 sm:p-6">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <!-- Before -->
                        <div>
                            <span class="block text-sm font-medium text-slate-700 mb-2">Before</span>
                            <div class="aspect-video w-full rounded-lg bg-slate-100 overflow-hidden border-2 border-dashed border-red-200 relative group">
                                <img 
                                  src={report.photo_before} 
                                  alt="Before" 
                                  class="w-full h-full object-cover"
                                  loading="lazy"
                                  decoding="async"
                                />
                            </div>
                        </div>

                        <!-- After -->
                        <div>
                            <span class="block text-sm font-medium text-slate-700 mb-2">After</span>
                            <div class="aspect-video w-full rounded-lg bg-slate-100 overflow-hidden border-2 border-dashed border-green-200 relative group">
                                {#if report.photo_after}
                                    <img 
                                      src={report.photo_after} 
                                      alt="After" 
                                      class="w-full h-full object-cover"
                                      loading="lazy"
                                      decoding="async"
                                    />
                                {:else}
                                    <div class="w-full h-full flex items-center justify-center text-slate-400">
                                        No Photo
                                    </div>
                                {/if}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="bg-slate-50 px-4 py-4 sm:px-6 flex justify-end gap-3">
                    <button 
                        onclick={() => handleVerify(report.id, 'deny')}
                        class="inline-flex items-center px-4 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-red-50 hover:text-red-700 hover:border-red-300 focus:outline-none"
                    >
                        Deny Claim
                    </button>
                    <button 
                        onclick={() => handleVerify(report.id, 'accept')}
                        class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none"
                    >
                        Verify & Award Points
                    </button>
                </div>
            </div>
        {/each}

        {#if queue.length === 0}
            <div class="text-center py-12">
                <span class="text-4xl block mb-4">🎉</span>
                <h3 class="text-lg font-medium text-slate-900">All caught up!</h3>
                <p class="mt-1 text-slate-500">There are no pending cleanups to verify nearby right now.</p>
            </div>
        {/if}
        </div>
    {/if}

  </div>
</div>
