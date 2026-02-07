<script lang="ts">
  let isSubmitting = false;

  function handleSubmit(e: Event) {
    e.preventDefault();
    isSubmitting = true;
    // Simulate API call
    setTimeout(() => {
        isSubmitting = false;
        alert('Report submitted successfully! (Mock)');
        window.location.href = '/app/feed';
    }, 1500);
  }
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="md:flex md:items-center md:justify-between mb-8">
      <div class="flex-1 min-w-0">
        <h2 class="text-2xl font-bold leading-7 text-slate-900 sm:text-3xl sm:truncate">
          Report Litter
        </h2>
      </div>
    </div>

    <div class="bg-white shadow overflow-hidden sm:rounded-lg border border-slate-200">
      <div class="px-4 py-5 sm:p-6">
        <form onsubmit={handleSubmit} class="space-y-6">
          
          <!-- Photo Upload -->
          <div>
            <label for="photo" class="block text-sm font-medium text-slate-700">Photo Evidence</label>
            <div class="mt-1 flex justify-center px-6 pt-5 pb-6 border-2 border-slate-300 border-dashed rounded-md hover:border-primary-500 transition-colors cursor-pointer group">
              <div class="space-y-1 text-center">
                <svg class="mx-auto h-12 w-12 text-slate-400 group-hover:text-primary-500" stroke="currentColor" fill="none" viewBox="0 0 48 48" aria-hidden="true">
                  <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                <div class="flex text-sm text-slate-600">
                  <span class="relative cursor-pointer bg-white rounded-md font-medium text-primary-600 hover:text-primary-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-primary-500">
                    <span>Upload a file</span>
                  </span>
                  <p class="pl-1">or drag and drop</p>
                </div>
                <p class="text-xs text-slate-500">PNG, JPG, GIF up to 10MB</p>
              </div>
            </div>
          </div>

          <!-- Location -->
          <div>
            <label for="location" class="block text-sm font-medium text-slate-700">Location Name / Description</label>
            <div class="mt-1">
              <input type="text" name="location" id="location" class="shadow-sm focus:ring-primary-500 focus:border-primary-500 block w-full sm:text-sm border-slate-300 rounded-md px-3 py-2" placeholder="e.g. Behind the bus stop on Main St">
            </div>
          </div>
          
           <!-- Coordinates (Mock hidden/auto-filled) -->
           <div>
              <button type="button" class="inline-flex items-center px-3 py-2 border border-slate-300 shadow-sm text-sm leading-4 font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50 focus:outline-none">
                 📍 Use Current Location (GPS)
              </button>
              <p class="mt-2 text-xs text-slate-500">Accurate within 10 meters.</p>
           </div>

          <!-- Description -->
          <div>
            <label for="description" class="block text-sm font-medium text-slate-700">Additional Notes</label>
            <div class="mt-1">
              <textarea id="description" name="description" rows="3" class="shadow-sm focus:ring-primary-500 focus:border-primary-500 block w-full sm:text-sm border border-slate-300 rounded-md px-3 py-2" placeholder="Describe the type of litter or any hazards..."></textarea>
            </div>
          </div>

          <!-- Submit -->
          <div class="pt-5">
            <div class="flex justify-end">
              <a href="/app/feed" class="bg-white py-2 px-4 border border-slate-300 rounded-md shadow-sm text-sm font-medium text-slate-700 hover:bg-slate-50 focus:outline-none">
                Cancel
              </a>
              <button type="submit" disabled={isSubmitting} class="ml-3 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50">
                {isSubmitting ? 'Submitting...' : 'Submit Report'}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  </div>
</div>
