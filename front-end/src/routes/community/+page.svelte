<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type LeaderboardEntry } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import Feed from '$lib/components/Feed.svelte';

  let entries: LeaderboardEntry[] = [];
  let period: 'weekly' | 'monthly' | 'all_time' = 'weekly';
  let loading = true;
  let error = '';
    let activeTab: 'leaderboard' | 'feed' = 'feed';

  async function loadLeaderboard() {
    loading = true;
    error = '';
    try {
        const data = await api.leaderboards.getGlobal(period, $auth.token ?? undefined);
        entries = data;
    } catch (e: any) {
        error = e.message || 'Failed to load leaderboard';
    } finally {
        loading = false;
    }
  }

  function changePeriod(newPeriod: 'weekly' | 'monthly' | 'all_time') {
      period = newPeriod;
      loadLeaderboard();
  }

    onMount(() => {
            loadLeaderboard();
    });
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="md:flex md:items-center md:justify-between mb-8">
      <div class="flex-1 min-w-0">
        <h2 class="text-2xl font-bold leading-7 text-slate-900 sm:text-3xl sm:truncate">
          Community
        </h2>
        <p class="mt-1 text-sm text-slate-500">
          Connect, share, and see who's making a difference.
        </p>
      </div>
    </div>

    <div class="mb-6 border-b border-slate-200">
        <div class="flex -mb-px">
            <button
                on:click={() => activeTab = 'leaderboard'}
                class={`py-3 px-5 font-medium text-sm transition-colors ${
                    activeTab === 'leaderboard'
                        ? 'border-b-2 border-primary-500 text-primary-600'
                        : 'text-slate-500 hover:text-slate-700'
                }`}
            >
                Leaderboard
            </button>
            <button
                on:click={() => activeTab = 'feed'}
                class={`py-3 px-5 font-medium text-sm transition-colors ${
                    activeTab === 'feed'
                        ? 'border-b-2 border-primary-500 text-primary-600'
                        : 'text-slate-500 hover:text-slate-700'
                }`}
            >
                Activity Feed
            </button>
        </div>
    </div>

    {#if activeTab === 'leaderboard'}
        <div class="md:flex md:items-center md:justify-end mb-8">
          <div class="mt-4 flex md:mt-0 bg-white rounded-lg p-1 shadow-sm border border-slate-200">
            <button 
                on:click={() => changePeriod('weekly')}
                class={`px-3 py-1 text-sm font-medium rounded-md transition-colors ${period === 'weekly' ? 'bg-primary-100 text-primary-700' : 'text-slate-500 hover:text-slate-900'}`}
            >
                Weekly
            </button>
            <button 
                on:click={() => changePeriod('monthly')}
                class={`px-3 py-1 text-sm font-medium rounded-md transition-colors ${period === 'monthly' ? 'bg-primary-100 text-primary-700' : 'text-slate-500 hover:text-slate-900'}`}
            >
                Monthly
            </button>
            <button 
                on:click={() => changePeriod('all_time')}
                class={`px-3 py-1 text-sm font-medium rounded-md transition-colors ${period === 'all_time' ? 'bg-primary-100 text-primary-700' : 'text-slate-500 hover:text-slate-900'}`}
            >
                All Time
            </button>
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
        {:else if entries.length === 0}
            <div class="text-center py-12 bg-white rounded-lg shadow border border-slate-200">
                <span class="text-4xl block mb-4">🏆</span>
                <h3 class="text-lg font-medium text-slate-900">No entries yet</h3>
                <p class="mt-2 text-slate-500">Be the first to score points this period!</p>
            </div>
        {:else}
            <div class="bg-white shadow overflow-hidden sm:rounded-lg border border-slate-200">
                <ul class="divide-y divide-slate-200">
                    {#each entries as entry}
                        <li class="px-4 py-4 sm:px-6 hover:bg-slate-50 transition-colors">
                            <div class="flex items-center justify-between">
                                <div class="flex items-center min-w-0 gap-4">
                                    <div class="shrink-0 w-8 h-8 flex items-center justify-center font-bold text-lg text-slate-400">
                                        #{entry.rank}
                                    </div>
                                    <div class="shrink-0">
                                        <div class="h-10 w-10 rounded-full bg-slate-200 flex items-center justify-center text-slate-500 font-bold">
                                            {entry.full_name.charAt(0)}
                                        </div>
                                    </div>
                                    <div class="min-w-0">
                                        <p class="text-sm font-medium text-primary-600 truncate">
                                            {entry.full_name}
                                        </p>
                                        <p class="text-xs text-slate-500 truncate">
                                            {entry.city}, {entry.country}
                                        </p>
                                    </div>
                                </div>
                                <div class="flex flex-col items-end gap-1">
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                                        {entry.total_points} pts
                                    </span>
                                    <span class="text-xs text-slate-500">
                                        {entry.reports_cleared} cleans
                                    </span>
                                </div>
                            </div>
                        </li>
                    {/each}
                </ul>
            </div>
        {/if}
    {:else if activeTab === 'feed'}
        <Feed />
    {/if}
  </div>
</div>
