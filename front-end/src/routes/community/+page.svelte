<script lang="ts">
  import { communityPosts } from '$lib/mockData';
</script>

<div class="bg-slate-50 min-h-full py-8">
  <div class="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8">
    
    <div class="md:flex md:items-center md:justify-between mb-8">
      <div class="flex-1 min-w-0">
        <h2 class="text-2xl font-bold leading-7 text-slate-900 sm:text-3xl sm:truncate">
          Community
        </h2>
        <p class="mt-1 text-sm text-slate-500">
          See what others are accomplishing around the world.
        </p>
      </div>
      <div class="mt-4 flex md:mt-0 md:ml-4">
        <button type="button" class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500">
          Share Update
        </button>
      </div>
    </div>

    <div class="space-y-6">
      {#each communityPosts as post}
        <div class="bg-white shadow overflow-hidden sm:rounded-lg border border-slate-200">
          <div class="p-6">
            <div class="flex space-x-3">
              <div class="flex-shrink-0">
                <div class={`h-10 w-10 rounded-full flex items-center justify-center text-white text-sm font-bold ${post.avatar}`}>
                    {post.user.charAt(0)}
                </div>
              </div>
              <div class="min-w-0 flex-1">
                <p class="text-sm font-medium text-slate-900">
                  <a href="#" class="hover:underline">{post.user}</a>
                  <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-slate-100 text-slate-800 ml-2">
                    {post.userTier}
                  </span>
                </p>
                <p class="text-sm text-slate-500">
                  <a href="#" class="hover:underline">{post.time}</a>
                </p>
              </div>
            </div>
            
            <p class="mt-4 text-base text-slate-900">
                {post.content}
            </p>
          </div>
          
          <!-- Post Image (Mock) -->
          <div class={`h-64 w-full bg-slate-200 flex items-center justify-center text-slate-400 ${post.image}`}>
             <span class="text-5xl">📸</span>
          </div>

          <div class="bg-slate-50 px-6 py-4 border-t border-slate-100">
            <div class="flex space-x-6">
                <button class="flex items-center space-x-2 text-sm text-slate-500 hover:text-primary-600 transition-colors">
                    <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 10h4.764a2 2 0 011.789 2.894l-3.5 7A2 2 0 0115.263 21h-4.017c-.163 0-.326-.02-.485-.06L7 20m7-10V5a2 2 0 00-2-2h-.095c-.5 0-.905.405-.905.905 0 .714-.211 1.412-.608 2.006L7 11v9m7-10h-2M7 20H5a2 2 0 01-2-2v-6a2 2 0 012-2h2.5" />
                    </svg>
                    <span>{post.likes} Likes</span>
                </button>
                <button class="flex items-center space-x-2 text-sm text-slate-500 hover:text-primary-600 transition-colors">
                    <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                    </svg>
                    <span>{post.comments} Comments</span>
                </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
