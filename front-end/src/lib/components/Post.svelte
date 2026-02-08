<script lang="ts">
    import { auth } from '$lib/stores/auth';
    import CommentSection from './CommentSection.svelte';

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

    export let post: FeedPostResponse;

    let isLiked = false;
    let showComments = false;
    let isLoadingLike = false;
    let currentImageIndex = 0;
    let isEditing = false;
    let editContent = post.content;

    async function toggleLike() {
        if (!$auth.token) return;

        isLoadingLike = true;
        try {
            const endpoint = isLiked ? `/api/feed/${post.id}/like` : `/api/feed/${post.id}/like`;
            const method = isLiked ? 'DELETE' : 'POST';

            const response = await fetch(endpoint, {
                method,
                headers: {
                    'Authorization': `Bearer ${$auth.token}`,
                },
            });

            if (response.ok) {
                isLiked = !isLiked;
                post.like_count += isLiked ? 1 : -1;
            }
        } catch (error) {
            console.error('Failed to toggle like:', error);
        } finally {
            isLoadingLike = false;
        }
    }

    async function handlePostDelete() {
        if (!confirm('Are you sure you want to delete this post?')) return;
        if (!$auth.token) return;

        try {
            const response = await fetch(`/api/feed/${post.id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${$auth.token}`,
                },
            });

            if (response.ok) {
                window.dispatchEvent(new CustomEvent('post-deleted', { detail: { postId: post.id } }));
            }
        } catch (error) {
            console.error('Failed to delete post:', error);
        }
    }

    function previousImage() {
        currentImageIndex = (currentImageIndex - 1 + post.images.length) % post.images.length;
    }

    function nextImage() {
        currentImageIndex = (currentImageIndex + 1) % post.images.length;
    }
</script>

<div class="bg-white border border-slate-200 rounded-lg p-4 mb-4 shadow-sm">
    <!-- Header with author info -->
    <div class="flex items-center justify-between mb-3">
        <div class="flex items-center">
            <div class="w-10 h-10 rounded-full bg-slate-200 flex items-center justify-center text-slate-500 font-bold mr-3">
                {post.author_name.charAt(0).toUpperCase()}
            </div>
            <div>
                <p class="font-bold text-slate-800">{post.author_name}</p>
                <p class="text-xs text-slate-500">
                    {new Date(post.created_at).toLocaleDateString()}
                </p>
            </div>
        </div>

        {#if post.user_id === $auth.user?.id}
            <div class="flex gap-2">
                <button
                    on:click={() => (isEditing = !isEditing)}
                    class="text-slate-500 hover:text-primary-600 text-sm"
                >
                    ✏️
                </button>
                <button
                    on:click={handlePostDelete}
                    class="text-slate-500 hover:text-red-600 text-sm"
                >
                    🗑️
                </button>
            </div>
        {/if}
    </div>

    <!-- Content -->
    <p class="text-slate-700 mb-3 whitespace-pre-wrap">{post.content}</p>

    <!-- Images carousel -->
    {#if post.images.length > 0}
        <div class="mb-3 relative">
            <img
                src={post.images[currentImageIndex]}
                alt="Post image {currentImageIndex + 1}"
                class="rounded-lg max-w-full h-auto"
            />

            {#if post.images.length > 1}
                <button
                    on:click={previousImage}
                    class="absolute left-2 top-1/2 -translate-y-1/2 bg-black/50 hover:bg-black/70 text-white px-2 py-1 rounded text-sm"
                >
                    ❮
                </button>
                <button
                    on:click={nextImage}
                    class="absolute right-2 top-1/2 -translate-y-1/2 bg-black/50 hover:bg-black/70 text-white px-2 py-1 rounded text-sm"
                >
                    ❯
                </button>
                <p class="text-xs text-slate-500 text-center mt-1">
                    {currentImageIndex + 1} / {post.images.length}
                </p>
            {/if}
        </div>
    {/if}

    <!-- Actions bar -->
    <div class="flex justify-around border-t border-slate-200 pt-3">
        <button
            on:click={toggleLike}
            disabled={isLoadingLike || !$auth.isAuthenticated}
            class="flex items-center gap-1 text-slate-500 hover:text-primary-600 font-medium text-sm py-1 px-3 rounded-md transition-colors disabled:opacity-50"
        >
            {isLiked ? '❤️' : '🤍'} {post.like_count}
        </button>
        <button
            on:click={() => (showComments = !showComments)}
            class="flex items-center gap-1 text-slate-500 hover:text-primary-600 font-medium text-sm py-1 px-3 rounded-md transition-colors"
        >
            💬 {post.comment_count}
        </button>
    </div>

    <!-- Comments section -->
    {#if showComments}
        <div class="mt-4 pt-4 border-t border-slate-200">
            <CommentSection {post} />
        </div>
    {/if}
</div>
