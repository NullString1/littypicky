<script lang="ts">
  import { currentUser } from '$lib/mockData';
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    
    <!-- Header Card -->
    <div class="bg-white rounded-lg shadow overflow-hidden border border-slate-200 mb-6">
      <div class="h-32 bg-gradient-to-r from-primary-500 to-primary-600"></div>
      <div class="relative px-4 pb-6 sm:px-6 lg:px-8">
        <div class="-mt-12 sm:-mt-16 sm:flex sm:items-end sm:space-x-5">
          <div class="flex">
            <div class={`h-24 w-24 rounded-full ring-4 ring-white flex items-center justify-center text-white text-3xl font-bold sm:h-32 sm:w-32 ${currentUser.avatar}`}>
               {currentUser.name.charAt(0)}
            </div>
          </div>
          <div class="mt-6 sm:flex-1 sm:min-w-0 sm:flex sm:items-center sm:justify-end sm:space-x-6 sm:pb-1">
            <div class="sm:hidden md:block mt-6 min-w-0 flex-1">
              <h1 class="text-2xl font-bold text-slate-900 truncate">
                {currentUser.name}
              </h1>
              <p class="text-sm font-medium text-slate-500">{currentUser.handle}</p>
            </div>
            <div class="mt-6 flex flex-col justify-stretch space-y-3 sm:flex-row sm:space-y-0 sm:space-x-4">
               <span class="inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium bg-primary-100 text-primary-800">
                 Tier: {currentUser.tier}
               </span>
              <button type="button" class="inline-flex justify-center px-4 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50 focus:outline-none">
                Edit Profile
              </button>
            </div>
          </div>
        </div>
        <div class="hidden sm:block md:hidden mt-6 min-w-0 flex-1">
          <h1 class="text-2xl font-bold text-slate-900 truncate">
            {currentUser.name}
          </h1>
          <p class="text-sm font-medium text-slate-500">{currentUser.handle}</p>
        </div>
      </div>
      
      <!-- Stats -->
      <div class="border-t border-slate-200 bg-slate-50 grid grid-cols-1 divide-y divide-slate-200 sm:grid-cols-3 sm:divide-y-0 sm:divide-x">
        <div class="px-6 py-5 text-center text-sm font-medium">
          <span class="text-slate-900 text-2xl font-bold block">{currentUser.points}</span>
          <span class="text-slate-500">Total Points</span>
        </div>
        <div class="px-6 py-5 text-center text-sm font-medium">
          <span class="text-slate-900 text-2xl font-bold block">{currentUser.cleanups}</span>
          <span class="text-slate-500">Cleanups Verified</span>
        </div>
        <div class="px-6 py-5 text-center text-sm font-medium">
          <span class="text-slate-900 text-2xl font-bold block">{currentUser.reports}</span>
          <span class="text-slate-500">Spots Reported</span>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
       <!-- About / Details -->
       <div class="bg-white shadow rounded-lg border border-slate-200 p-6">
          <h3 class="text-lg font-medium leading-6 text-slate-900 mb-4">About</h3>
          <p class="text-slate-600">
             Passionate about keeping our local parks and beaches clean. Weekend warrior for the environment. Always looking for the next cleanup spot!
          </p>
          
          <div class="mt-6 border-t border-slate-100 pt-4 space-y-3">
             <div class="flex justify-between text-sm">
                <span class="text-slate-500">Joined</span>
                <span class="font-medium text-slate-900">March 2024</span>
             </div>
             <div class="flex justify-between text-sm">
                <span class="text-slate-500">Location</span>
                <span class="font-medium text-slate-900">New York, USA</span>
             </div>
          </div>
       </div>

       <!-- Badges -->
       <div class="lg:col-span-2 bg-white shadow rounded-lg border border-slate-200 p-6">
          <h3 class="text-lg font-medium leading-6 text-slate-900 mb-4">Badges & Achievements</h3>
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
             {#each currentUser.badges as badge}
                <div class="relative rounded-lg border border-slate-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-slate-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-primary-500">
                  <div class="flex-shrink-0 text-3xl">
                     {badge.icon}
                  </div>
                  <div class="flex-1 min-w-0">
                    <a href="#" class="focus:outline-none">
                      <span class="absolute inset-0" aria-hidden="true"></span>
                      <p class="text-sm font-medium text-slate-900">{badge.name}</p>
                      <p class="text-sm text-slate-500 truncate">{badge.description}</p>
                    </a>
                  </div>
                </div>
             {/each}
             
             <!-- Locked Badge Example -->
             <div class="relative rounded-lg border border-slate-200 bg-slate-50 px-6 py-5 shadow-sm flex items-center space-x-3 opacity-60 grayscale">
                <div class="flex-shrink-0 text-3xl">
                   👑
                </div>
                <div class="flex-1 min-w-0">
                  <div class="focus:outline-none">
                    <p class="text-sm font-medium text-slate-900">Litter Legend</p>
                    <p class="text-sm text-slate-500 truncate">Complete 100 cleanups</p>
                  </div>
                </div>
                <div class="absolute top-2 right-2 text-xs font-bold text-slate-400">LOCKED</div>
             </div>
          </div>
       </div>
    </div>

  </div>
</div>
