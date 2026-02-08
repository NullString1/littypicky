<script lang="ts">
    import { auth } from '$lib/stores/auth';

    interface Comment {
        id: string;
        post_id: string;
        user_id: string | null;
        author_name: string | null;
        content: string;
        is_deleted: boolean;
        created_at: string;
        updated_at: string;
    }

    interface FeedPostResponse {
        id: string;
        comments: Comment[];
    }

    export let post: FeedPostResponse;

    let comments: Comment[] = post.comments;
    let newCommentContent = '';
    let isSubmittingComment = false;
    let error = '';
    let editingCommentId: string | null = null;
    let editingContent = '';

    async function submitComment() {
        if (!newCommentContent.trim()) return;
        if (!$auth.token) return;

        isSubmittingComment = true;
        error = '';

        try {
            const response = await fetch(`/api/feed/${post.id}/comments`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${$auth.token}`,
                },
                body: JSON.stringify({ content: newCommentContent.trim() }),
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to add comment');
            }

            const comment: Comment = await response.json();
            comment.author_name = $auth.user?.full_name || 'Unknown';
            comments = [...comments, comment];
            newCommentContent = '';
        } catch (e: any) {
            error = e.message || 'Failed to add comment';
        } finally {
            isSubmittingComment = false;
        }
    }

    async function deleteComment(commentId: string) {
        if (!confirm('Delete this comment?')) return;
        if (!$auth.token) return;

        try {
            const response = await fetch(`/api/feed/comments/${commentId}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${$auth.token}`,
                },
            });

            if (response.ok) {
                comments = comments.map((c) =>
                    c.id === commentId ? { ...c, is_deleted: true, content: '[deleted]', user_id: null, author_name: null } : c
                );
            }
        } catch (error) {
            console.error('Failed to delete comment:', error);
        }
    }

    async function startEditComment(comment: Comment) {
        editingCommentId = comment.id;
        editingContent = comment.content;
    }

    async function saveEditComment(commentId: string) {
        if (!editingContent.trim()) return;
        if (!$auth.token) return;

        try {
            const response = await fetch(`/api/feed/comments/${commentId}`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${$auth.token}`,
                },
                body: JSON.stringify({ content: editingContent.trim() }),
            });

            if (response.ok) {
                const updated: Comment = await response.json();
                comments = comments.map((c) => (c.id === commentId ? updated : c));
                editingCommentId = null;
                editingContent = '';
            }
        } catch (error) {
            console.error('Failed to update comment:', error);
        }
    }

    function cancelEditComment() {
        editingCommentId = null;
        editingContent = '';
    }
</script>

<div class="space-y-3">
    {#if error}
        <div class="bg-red-50 border border-red-200 text-red-600 px-3 py-2 rounded-md text-sm">
            {error}
        </div>
    {/if}

    <!-- Comment list -->
    {#if comments.length > 0}
        <div class="space-y-2 max-h-64 overflow-y-auto">
            {#each comments as comment (comment.id)}
                <div class="bg-slate-50 p-3 rounded-md text-sm">
                    <div class="flex justify-between items-start">
                        <div>
                            <p class="font-semibold text-slate-800">{comment.author_name || '[deleted user]'}</p>
                            <p class="text-xs text-slate-500">
                                {new Date(comment.created_at).toLocaleDateString()}
                            </p>
                        </div>

                        {#if !comment.is_deleted && comment.user_id === $auth.user?.id}
                            <div class="flex gap-1">
                                <button
                                    on:click={() => startEditComment(comment)}
                                    class="text-xs text-slate-500 hover:text-primary-600"
                                >
                                    Edit
                                </button>
                                <button
                                    on:click={() => deleteComment(comment.id)}
                                    class="text-xs text-slate-500 hover:text-red-600"
                                >
                                    Delete
                                </button>
                            </div>
                        {/if}
                    </div>

                    {#if editingCommentId === comment.id}
                        <textarea
                            bind:value={editingContent}
                            class="w-full mt-2 p-2 border border-slate-300 rounded-md text-sm"
                            rows="2"
                        ></textarea>
                        <div class="flex gap-2 mt-2">
                            <button
                                on:click={() => saveEditComment(comment.id)}
                                class="text-xs bg-primary-600 text-white px-2 py-1 rounded hover:bg-primary-700"
                            >
                                Save
                            </button>
                            <button
                                on:click={cancelEditComment}
                                class="text-xs bg-slate-300 text-slate-800 px-2 py-1 rounded hover:bg-slate-400"
                            >
                                Cancel
                            </button>
                        </div>
                    {:else}
                        <p class="mt-1 text-slate-700 whitespace-pre-wrap">{comment.content}</p>
                    {/if}
                </div>
            {/each}
        </div>
    {:else}
        <p class="text-sm text-slate-500 text-center py-2">No comments yet</p>
    {/if}

    <!-- Add comment form -->
    <div class="mt-3 pt-3 border-t border-slate-200">
        {#if $auth.isAuthenticated}
            <textarea
                bind:value={newCommentContent}
                placeholder="Add a comment..."
                class="w-full p-2 border border-slate-300 rounded-md text-sm focus:ring-2 focus:ring-primary-300"
                rows="2"
                disabled={isSubmittingComment}
            ></textarea>
            <button
                on:click={submitComment}
                disabled={isSubmittingComment || !newCommentContent.trim()}
                class="w-full mt-2 bg-primary-600 text-white font-medium py-1 px-3 rounded-md hover:bg-primary-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
            >
                {isSubmittingComment ? 'Posting...' : 'Comment'}
            </button>
        {:else}
            <div class="text-sm text-slate-500 text-center py-2">
                Sign in to add a comment
            </div>
        {/if}
    </div>
</div>
