<script lang="ts">
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import imageCompression from 'browser-image-compression';
  import { getCurrentLocation, reverseGeocode } from '$lib/utils/geolocation';

  let isSubmitting = false;
  let photoPreview: string | null = null;
  let photoBase64: string | null = null;
  let compressionStatus = '';
  
  let latitude: number | null = null;
  let longitude: number | null = null;
  let city = '';
  let country = '';
  let locationStatus = '';
  let error = '';

  async function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      const file = input.files[0];
      
      // Limit original size to 10MB (will compress down)
      if (file.size > 10 * 1024 * 1024) {
          error = 'File size must be less than 10MB';
          return;
      }

      error = '';
      compressionStatus = 'Compressing image...';

      try {
        const originalSize = (file.size / 1024 / 1024).toFixed(2);
        
        // Compression options
        const options = {
          maxSizeMB: 1,              // Maximum size in MB
          maxWidthOrHeight: 1920,    // Maximum width or height
          useWebWorker: true,        // Use web worker for better performance
          fileType: 'image/jpeg'     // Convert to JPEG
        };

        // Compress image
        const compressedFile = await imageCompression(file, options);
        const compressedSize = (compressedFile.size / 1024 / 1024).toFixed(2);
        const reduction = (((file.size - compressedFile.size) / file.size) * 100).toFixed(0);
        
        compressionStatus = `Compressed ${originalSize}MB → ${compressedSize}MB (${reduction}% reduction)`;

        // Convert to base64
        const reader = new FileReader();
        reader.onload = (e) => {
          photoPreview = e.target?.result as string;
          photoBase64 = photoPreview;
        };
        reader.readAsDataURL(compressedFile);
      } catch (err: any) {
        error = err.message || 'Failed to process image';
        compressionStatus = '';
      }
    }
  }

  async function getLocation() {
    locationStatus = 'Locating...';
    
    try {
      // Get coordinates using utility
      const coords = await getCurrentLocation();
      latitude = coords.lat;
      longitude = coords.lng;
      locationStatus = 'Found coordinates. Fetching address...';

      // Reverse geocode using utility
      const address = await reverseGeocode(latitude, longitude);
      if (address) {
        city = address.city;
        country = address.country;
        locationStatus = `Located: ${city}, ${country}`;
      } else {
        locationStatus = 'Coordinates found. Please enter city manually.';
      }
    } catch (err: any) {
      locationStatus = 'Location access failed. Please enter manually.';
      console.error('Location error:', err);
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    
    if (!photoBase64) {
        error = 'Please upload a photo.';
        return;
    }
    if (!latitude || !longitude) {
        error = 'Please set the location.';
        return;
    }
    if (!city || !country) {
        error = 'City and Country are required.';
        return;
    }

    isSubmitting = true;
    
    const form = e.target as HTMLFormElement;
    const formData = new FormData(form);
    const description = formData.get('description') as string;

    try {
        if (!$auth.token) throw new Error('Not authenticated');

        await api.reports.create({
            city,
            country,
            description,
            latitude,
            longitude,
            photo_base64: photoBase64
        }, $auth.token);

        alert('Report submitted successfully!');
        goto('/app/feed');
    } catch (e: any) {
        error = e.message || 'Failed to submit report';
    } finally {
        isSubmitting = false;
    }
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
        {#if error}
            <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6">
                {error}
            </div>
        {/if}

        <form onsubmit={handleSubmit} class="space-y-6">
          
          <!-- Photo Upload -->
          <div>
            <label for="photo" class="block text-sm font-medium text-slate-700">Photo Evidence</label>
            {#if compressionStatus}
              <p class="mt-1 text-xs text-green-600">{compressionStatus}</p>
            {/if}
            <div class="mt-1 flex justify-center px-6 pt-5 pb-6 border-2 border-slate-300 border-dashed rounded-md hover:border-primary-500 transition-colors cursor-pointer group relative">
              <input id="photo" name="photo" type="file" accept="image/*" class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10" onchange={handleFileSelect} />
              
              <div class="space-y-1 text-center">
                {#if photoPreview}
                    <img src={photoPreview} alt="Preview" class="mx-auto h-48 object-contain" />
                {:else}
                    <svg class="mx-auto h-12 w-12 text-slate-400 group-hover:text-primary-500" stroke="currentColor" fill="none" viewBox="0 0 48 48" aria-hidden="true">
                    <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                    </svg>
                    <div class="flex text-sm text-slate-600 justify-center">
                    <span class="relative bg-white rounded-md font-medium text-primary-600 hover:text-primary-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-primary-500">
                        <span>Upload a file</span>
                    </span>
                    <p class="pl-1">or drag and drop</p>
                    </div>
                    <p class="text-xs text-slate-500">PNG, JPG, GIF up to 10MB (auto-compressed)</p>
                {/if}
              </div>
            </div>
          </div>

          <!-- Location -->
          <div>
            <label for="city" class="block text-sm font-medium text-slate-700">Location</label>
             <div class="mt-2 flex items-center gap-4">
               <button type="button" onclick={getLocation} class="inline-flex items-center px-3 py-2 border border-slate-300 shadow-sm text-sm leading-4 font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50 focus:outline-none">
                  📍 Use Current Location
               </button>
               <span class="text-sm text-slate-500">{locationStatus}</span>
             </div>
             
             <div class="grid grid-cols-2 gap-4 mt-4">
                <div>
                    <label for="city" class="block text-xs font-medium text-slate-500">City</label>
                    <input type="text" name="city" id="city" bind:value={city} required class="mt-1 shadow-sm focus:ring-primary-500 focus:border-primary-500 block w-full sm:text-sm border-slate-300 rounded-md px-3 py-2">
                </div>
                <div>
                    <label for="country" class="block text-xs font-medium text-slate-500">Country</label>
                    <input type="text" name="country" id="country" bind:value={country} required class="mt-1 shadow-sm focus:ring-primary-500 focus:border-primary-500 block w-full sm:text-sm border-slate-300 rounded-md px-3 py-2">
                </div>
             </div>
          </div>

          <!-- Description -->
          <div>
            <label for="description" class="block text-sm font-medium text-slate-700">Description</label>
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
