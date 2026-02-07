<script lang="ts">
  import { verificationQueue } from '$lib/mockData';

  function handleVerify(id: string, decision: 'accept' | 'deny') {
    alert(`Claim ${id} ${decision === 'accept' ? 'ACCEPTED' : 'DENIED'}. Points updated!`);
    // In a real app, this would remove the item from the list
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
          Review cleanup claims from fellow pickers.
        </p>
      </div>
      <div class="mt-4 flex md:mt-0 md:ml-4">
          <span class="inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium bg-primary-100 text-primary-800">
            {verificationQueue.length} Pending
          </span>
      </div>
    </div>

    <div class="space-y-8">
      {#each verificationQueue as request}
        <div class="bg-white shadow overflow-hidden sm:rounded-lg border border-slate-200">
            <div class="px-4 py-5 border-b border-slate-200 sm:px-6">
                <h3 class="text-lg leading-6 font-medium text-slate-900">
                    Cleanup by {request.cleanerName}
                </h3>
                <p class="mt-1 text-sm text-slate-500">
                    Submitted on {request.timestamp.toLocaleDateString()} at {request.timestamp.toLocaleTimeString()}
                </p>
            </div>
            
            <div class="px-4 py-5 sm:p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <!-- Before -->
                    <div>
                        <span class="block text-sm font-medium text-slate-700 mb-2">Before</span>
                        <div class={`aspect-video w-full rounded-lg flex items-center justify-center text-slate-500 border-2 border-dashed border-red-200 ${request.beforePhotoUrl}`}>
                           <span class="text-lg font-semibold">Messy Photo</span>
                        </div>
                    </div>

                    <!-- After -->
                    <div>
                        <span class="block text-sm font-medium text-slate-700 mb-2">After</span>
                        <div class={`aspect-video w-full rounded-lg flex items-center justify-center text-slate-500 border-2 border-dashed border-green-200 ${request.afterPhotoUrl}`}>
                            <span class="text-lg font-semibold">Clean Photo</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="bg-slate-50 px-4 py-4 sm:px-6 flex justify-end gap-3">
                 <button 
                    onclick={() => handleVerify(request.id, 'deny')}
                    class="inline-flex items-center px-4 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-red-50 hover:text-red-700 hover:border-red-300 focus:outline-none"
                 >
                    Deny Claim
                 </button>
                 <button 
                    onclick={() => handleVerify(request.id, 'accept')}
                    class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none"
                 >
                    Verify & Award Points
                 </button>
            </div>
        </div>
      {/each}

      {#if verificationQueue.length === 0}
         <div class="text-center py-12">
             <span class="text-4xl block mb-4">🎉</span>
             <h3 class="text-lg font-medium text-slate-900">All caught up!</h3>
             <p class="mt-1 text-slate-500">There are no pending cleanups to verify right now.</p>
         </div>
      {/if}
    </div>

  </div>
</div>
