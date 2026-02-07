<script lang="ts">
  import { activeReports } from '$lib/mockData';

  // Helper to open Google Maps
  function openDirections(lat: number, lng: number) {
    window.open(`https://www.google.com/maps/dir/?api=1&destination=${lat},${lng}`, '_blank');
  }
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    
    <!-- Header -->
    <div class="md:flex md:items-center md:justify-between mb-8">
      <div class="flex-1 min-w-0">
        <h2 class="text-2xl font-bold leading-7 text-slate-900 sm:text-3xl sm:truncate">
          Litter Feed
        </h2>
        <p class="mt-1 text-sm text-slate-500">
          Find a messy spot near you and clean it up! First come, first serve.
        </p>
      </div>
      <div class="mt-4 flex md:mt-0 md:ml-4">
        <a href="/app/report" class="ml-3 inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500">
          Report New Spot
        </a>
      </div>
    </div>

    <!-- Feed Grid -->
    <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
      {#each activeReports as report}
        <div class="bg-white overflow-hidden shadow rounded-lg border border-slate-200 flex flex-col">
          <!-- Photo Placeholder -->
          <div class="h-48 w-full bg-slate-200 flex items-center justify-center text-slate-400">
             <span class="text-4xl">📸</span>
          </div>
          
          <div class="px-4 py-5 sm:p-6 flex-1 flex flex-col">
            <div class="flex items-center justify-between mb-2">
               <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                 {report.status}
               </span>
               <span class="text-xs text-slate-500">{report.distance} away</span>
            </div>
            
            <h3 class="text-lg leading-6 font-medium text-slate-900">
              {report.locationName}
            </h3>
            <p class="mt-2 text-sm text-slate-500 line-clamp-2">
              {report.description}
            </p>
            
            <div class="mt-6 flex-1 flex items-end">
                <button 
                  onclick={() => openDirections(report.coordinates.lat, report.coordinates.lng)}
                  class="w-full flex items-center justify-center px-4 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50"
                >
                    📍 Get Directions
                </button>
            </div>
          </div>
          <div class="bg-slate-50 px-4 py-4 sm:px-6 border-t border-slate-100">
            <div class="text-xs text-slate-500 flex justify-between items-center">
                <span>Reported by User #{report.reporterId}</span>
                <span>{report.timestamp.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}</span>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
