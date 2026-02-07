<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { api, type User, type UserScoreRecord } from '$lib/api';
  import { auth } from '$lib/stores/auth';

  let user: User | null = $auth.user;
  let score: UserScoreRecord | null = null;
  let loading = true;

  onMount(async () => {
    if (!$auth.token) return;
    try {
        const [userData, scoreData] = await Promise.all([
            api.auth.getMe($auth.token),
            api.users.getMyScore($auth.token)
        ]);
        user = userData;
        score = scoreData;
        // Update store
        if ($auth.token && user && $auth.refreshToken) {
            auth.login($auth.token, user, $auth.refreshToken);
        }
    } catch (e) {
        console.error('Failed to load profile:', e);
    } finally {
        loading = false;
    }
  });

  function getTier(points: number) {
      if (points < 100) return 'Novice Picker';
      if (points < 500) return 'Community Cleaner';
      if (points < 1000) return 'Eco Warrior';
      return 'Litter Legend';
  }
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    
    {#if user}
    <!-- Header Card -->
    <div class="bg-white rounded-lg shadow overflow-hidden border border-slate-200 mb-6">
      <div class="h-32 bg-gradient-to-r from-primary-500 to-primary-600"></div>
      <div class="relative px-4 pb-6 sm:px-6 lg:px-8">
        <div class="-mt-12 sm:-mt-16 sm:flex sm:items-end sm:space-x-5">
          <div class="flex">
            <div class="h-24 w-24 rounded-full ring-4 ring-white bg-slate-200 flex items-center justify-center text-slate-500 text-3xl font-bold sm:h-32 sm:w-32">
               {user.full_name.charAt(0)}
            </div>
          </div>
          <div class="mt-6 sm:flex-1 sm:min-w-0 sm:flex sm:items-center sm:justify-end sm:space-x-6 sm:pb-1">
            <div class="sm:hidden md:block mt-6 min-w-0 flex-1">
              <h1 class="text-2xl font-bold text-slate-900 truncate">
                {user.full_name}
              </h1>
              <p class="text-sm font-medium text-slate-500">{user.email}</p>
            </div>
            <div class="mt-6 flex flex-col justify-stretch space-y-3 sm:flex-row sm:space-y-0 sm:space-x-4">
               <span class="inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium bg-primary-100 text-primary-800">
                 {score ? getTier(score.total_points) : 'Loading...'}
               </span>
               <button type="button" onclick={() => goto('/profile/me/edit')} class="inline-flex justify-center px-4 py-2 border border-slate-300 shadow-sm text-sm font-medium rounded-md text-slate-700 bg-white hover:bg-slate-50 focus:outline-none">
                 Edit Profile
               </button>
            </div>
          </div>
        </div>
        <div class="hidden sm:block md:hidden mt-6 min-w-0 flex-1">
          <h1 class="text-2xl font-bold text-slate-900 truncate">
            {user.full_name}
          </h1>
          <p class="text-sm font-medium text-slate-500">{user.email}</p>
        </div>
      </div>
      
      <!-- Stats -->
      <div class="border-t border-slate-200 bg-slate-50 grid grid-cols-1 divide-y divide-slate-200 sm:grid-cols-4 sm:divide-y-0 sm:divide-x">
        <div class="px-6 py-5 text-center text-sm font-medium">
          <span class="text-slate-900 text-2xl font-bold block">{score?.total_points || 0}</span>
          <span class="text-slate-500">Total Points</span>
        </div>
        <div class="px-6 py-5 text-center text-sm font-medium">
          <span class="text-slate-900 text-2xl font-bold block">{score?.total_clears || 0}</span>
          <span class="text-slate-500">Cleanups</span>
        </div>
        <div class="px-6 py-5 text-center text-sm font-medium">
          <span class="text-slate-900 text-2xl font-bold block">{score?.total_reports || 0}</span>
          <span class="text-slate-500">Reports</span>
        </div>
        <div class="px-6 py-5 text-center text-sm font-medium">
            <span class="text-slate-900 text-2xl font-bold block">{score?.current_streak || 0}</span>
            <span class="text-slate-500">Day Streak</span>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
       <!-- About / Details -->
       <div class="bg-white shadow rounded-lg border border-slate-200 p-6">
          <h3 class="text-lg font-medium leading-6 text-slate-900 mb-4">About</h3>
          
          <div class="mt-6 border-t border-slate-100 pt-4 space-y-3">
             <div class="flex justify-between text-sm">
                <span class="text-slate-500">Joined</span>
                <span class="font-medium text-slate-900">{new Date(user.created_at).toLocaleDateString()}</span>
             </div>
             <div class="flex justify-between text-sm">
                <span class="text-slate-500">Location</span>
                <span class="font-medium text-slate-900">{user.city}, {user.country}</span>
             </div>
             <div class="flex justify-between text-sm">
                <span class="text-slate-500">Search Radius</span>
                <span class="font-medium text-slate-900">{user.search_radius_km} km</span>
             </div>
          </div>
       </div>

       <!-- Badges (Placeholder) -->
       <div class="lg:col-span-2 bg-white shadow rounded-lg border border-slate-200 p-6">
          <h3 class="text-lg font-medium leading-6 text-slate-900 mb-4">Achievements</h3>
          <div class="text-center py-8 text-slate-500">
              {#if (score?.total_clears || 0) > 0}
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 text-left">
                    {#if (score?.total_clears || 0) >= 1}
                        <div class="relative rounded-lg border border-slate-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3">
                            <div class="flex-shrink-0 text-3xl">🌱</div>
                            <div>
                                <p class="text-sm font-medium text-slate-900">First Steps</p>
                                <p class="text-xs text-slate-500">Completed 1 cleanup</p>
                            </div>
                        </div>
                    {/if}
                    {#if (score?.total_clears || 0) >= 10}
                        <div class="relative rounded-lg border border-slate-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3">
                            <div class="flex-shrink-0 text-3xl">🌿</div>
                            <div>
                                <p class="text-sm font-medium text-slate-900">Dedicated</p>
                                <p class="text-xs text-slate-500">Completed 10 cleanups</p>
                            </div>
                        </div>
                    {/if}
                    {#if (score?.current_streak || 0) >= 3}
                        <div class="relative rounded-lg border border-slate-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3">
                            <div class="flex-shrink-0 text-3xl">🔥</div>
                            <div>
                                <p class="text-sm font-medium text-slate-900">On Fire</p>
                                <p class="text-xs text-slate-500">3 day streak</p>
                            </div>
                        </div>
                    {/if}
                </div>
              {:else}
                <p>Start cleaning up litter to unlock achievements!</p>
              {/if}
          </div>
       </div>
    </div>
    {:else if loading}
        <div class="flex justify-center py-12">
            <div class="w-12 h-12 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
        </div>
    {/if}

  </div>
</div>
