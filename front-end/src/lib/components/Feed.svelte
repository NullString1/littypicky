<script lang="ts">
    import { onMount } from 'svelte';
    import Post from './Post.svelte';
    import PostForm from './PostForm.svelte';
    import { auth } from '$lib/stores/auth';

    interface FeedPostResponse {
        id: string;
        user_id: string;
        author_name: string;
        content: string;
        images: string[];
        like_count: number;
        comment_count: number;
        comments: any[];
        created_at: string;
        updated_at: string;
    }

    let posts: FeedPostResponse[] = [];
    let offset = 0;
    let limit = 20;
    let isLoading = false;
    let isLoadingMore = false;
    let hasMore = true;
    let error = '';

    async function loadFeed(append = false) {
        if (isLoading || isLoadingMore) return;
        if (!hasMore && append) return;

        const loaderState = append ? isLoadingMore : isLoading;
        if (append) {
            isLoadingMore = true;
        } else {
            isLoading = true;
        }

        error = '';

        try {
            const headers: Record<string, string> = {};
            if ($auth.token) {
                headers['Authorization'] = `Bearer ${$auth.token}`;
            }

            const response = await fetch(`/api/feed?offset=${offset}&limit=${limit}`, {
                headers,
            });

            if (!response.ok) {
                throw new Error('Failed to load feed');
            }

            const newPosts: FeedPostResponse[] = await response.json();

            if (append) {
                posts = [...posts, ...newPosts];
            } else {
                posts = newPosts;
            }

            // Update pagination
            if (newPosts.length < limit) {
                hasMore = false;
            }

            offset += limit;
        } catch (e: any) {
            error = e.message || 'Failed to load feed';
            hasMore = false;
        } finally {
            if (append) {
                isLoadingMore = false;
            } else {
                isLoading = false;
            }
        }
    }

    function handleScroll() {
        // Check if near bottom of page
        const scrollThreshold = 500; // pixels from bottom
        const position = window.innerHeight + window.scrollY;
        const bottom = document.body.offsetHeight;

        if (bottom - position < scrollThreshold && hasMore && !isLoadingMore) {
            loadFeed(true);
        }
    }

    function handlePostCreated(event: Event) {
        const customEvent = event as CustomEvent;
        offset = 0; // Reset pagination
        loadFeed(false);
    }

    function handlePostDeleted(event: Event) {
        const customEvent = event as CustomEvent;
        const postId = customEvent.detail?.postId;
        posts = posts.filter((p) => p.id !== postId);
    }

    onMount(() => {
        loadFeed();
        window.addEventListener('post-created', handlePostCreated);
        window.addEventListener('post-deleted', handlePostDeleted);
        window.addEventListener('scroll', handleScroll);

        return () => {
            window.removeEventListener('post-created', handlePostCreated);
            window.removeEventListener('post-deleted', handlePostDeleted);
            window.removeEventListener('scroll', handleScroll);
        };
    });
</script>

<div class="max-w-3xl mx-auto px-4">
    {#if $auth.isAuthenticated}
        <PostForm />
    {/if}

    {#if error}
        <div class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md mb-6">
            {error}
        </div>
    {/if}

    {#if isLoading}
        <div class="flex justify-center py-12">
            <div class="w-12 h-12 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
        </div>
    {:else if posts.length === 0}
        <div class="text-center py-12 bg-white rounded-lg shadow border border-slate-200">
            <span class="text-4xl block mb-4">📝</span>
            <h3 class="text-lg font-medium text-slate-900">No posts yet</h3>
            <p class="mt-2 text-slate-500">Be the first to share your community work!</p>
        </div>
    {:else}
        <div class="space-y-4">
            {#each posts as post (post.id)}
                <Post {post} />
            {/each}
        </div>

        {#if isLoadingMore}
            <div class="flex justify-center py-8">
                <div class="w-8 h-8 border-4 border-primary-200 border-t-primary-600 rounded-full animate-spin"></div>
            </div>
        {/if}

        {#if !hasMore && posts.length > 0}
            <div class="text-center py-8 text-slate-500">
                No more posts to load
            </div>
        {/if}
    {/if}
</div>

<style>
    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .animate-spin {
        animation: spin 1s linear infinite;
    }
</style>
