<script lang="ts">
    import { auth } from '$lib/stores/auth';

    interface CreateFeedPostRequest {
        content: string;
        images: string[];
    }

    interface FeedPostResponse {
        id: string;
        content: string;
        author_name: string;
        images: string[];
        like_count: number;
        comment_count: number;
    }

    let content = '';
    let selectedFiles: FileList | null = null;
    let imagePreviews: string[] = [];
    let isSubmitting = false;
    let error = '';

    async function handleFileSelect(event: Event) {
        const input = event.target as HTMLInputElement;
        if (!input.files) return;

        selectedFiles = input.files;
        imagePreviews = [];

        // Create previews and validate file sizes
        for (let i = 0; i < input.files.length; i++) {
            const file = input.files[i];

            // Check file size (5MB)
            if (file.size > 5 * 1024 * 1024) {
                error = `Image ${i + 1} is too large (max 5MB)`;
                selectedFiles = null;
                imagePreviews = [];
                return;
            }

            // Check max 10 images
            if (input.files.length > 10) {
                error = 'Maximum 10 images allowed';
                selectedFiles = null;
                imagePreviews = [];
                return;
            }

            // Create preview
            const reader = new FileReader();
            reader.onload = (e) => {
                if (e.target?.result) {
                    imagePreviews = [...imagePreviews, e.target.result as string];
                }
            };
            reader.readAsDataURL(file);
        }
        error = '';
    }

    function removeImage(index: number) {
        imagePreviews = imagePreviews.filter((_, i) => i !== index);

        // Update selectedFiles (create new FileList-like object)
        if (selectedFiles) {
            const dt = new DataTransfer();
            for (let i = 0; i < selectedFiles.length; i++) {
                if (i !== index) {
                    dt.items.add(selectedFiles[i]);
                }
            }
            selectedFiles = dt.files;
        }
    }

    async function handleSubmit() {
        if (!content.trim()) {
            error = 'Please enter some content';
            return;
        }

        error = '';
        isSubmitting = true;

        try {
            // Convert images to base64
            const images: string[] = [];
            if (selectedFiles) {
                for (let i = 0; i < selectedFiles.length; i++) {
                    const file = selectedFiles[i];
                    const base64 = await new Promise<string>((resolve, reject) => {
                        const reader = new FileReader();
                        reader.onload = () => resolve(reader.result as string);
                        reader.onerror = reject;
                        reader.readAsDataURL(file);
                    });
                    images.push(base64);
                }
            }

            const request: CreateFeedPostRequest = {
                content: content.trim(),
                images,
            };

            const token = $auth.token;
            if (!token) {
                error = 'Not authenticated';
                isSubmitting = false;
                return;
            }

            const response = await fetch('/api/feed', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`,
                },
                body: JSON.stringify(request),
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to create post');
            }

            const post: FeedPostResponse = await response.json();

            // Reset form
            content = '';
            selectedFiles = null;
            imagePreviews = [];
            const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
            if (fileInput) fileInput.value = '';

            // Dispatch event to refresh feed
            window.dispatchEvent(new CustomEvent('post-created', { detail: post }));
        } catch (e: any) {
            error = e.message || 'Failed to create post';
        } finally {
            isSubmitting = false;
        }
    }
</script>

<form on:submit|preventDefault={handleSubmit} class="bg-white border border-slate-200 rounded-lg p-4 mb-6 shadow-sm">
    {#if error}
        <div class="bg-red-50 border border-red-200 text-red-600 px-3 py-2 rounded-md mb-4 text-sm">
            {error}
        </div>
    {/if}

    <textarea
        class="w-full border border-slate-300 rounded-md p-2 text-slate-800 placeholder-slate-400 focus:ring-2 focus:ring-primary-300 focus:border-primary-500 transition"
        bind:value={content}
        placeholder="Share your community work..."
        rows="3"
        disabled={isSubmitting}
    ></textarea>

    <div class="mt-4">
        <label class="block text-sm font-medium text-slate-700 mb-2">
            Add Photos (Optional)
        </label>
        <input
            type="file"
            multiple
            accept="image/*"
            on:change={handleFileSelect}
            disabled={isSubmitting}
            class="block w-full text-sm text-slate-500
                file:mr-4 file:py-2 file:px-4
                file:rounded-md file:border-0
                file:text-sm file:font-semibold
                file:bg-primary-50 file:text-primary-700
                hover:file:bg-primary-100
                disabled:opacity-50"
        />
        <p class="text-xs text-slate-500 mt-1">Max 10 images, 5MB each</p>
    </div>

    {#if imagePreviews.length > 0}
        <div class="mt-4 grid grid-cols-3 gap-2">
            {#each imagePreviews as preview, index}
                <div class="relative">
                    <img
                        src={preview}
                        alt="Preview {index + 1}"
                        class="w-full h-24 object-cover rounded-md"
                    />
                    <button
                        type="button"
                        on:click={() => removeImage(index)}
                        disabled={isSubmitting}
                        class="absolute top-0 right-0 bg-red-500 text-white rounded-full p-1 text-xs hover:bg-red-600 disabled:opacity-50"
                    >
                        ✕
                    </button>
                </div>
            {/each}
        </div>
    {/if}

    <button
        type="submit"
        disabled={isSubmitting}
        class="w-full bg-primary-600 text-white font-bold py-2 px-4 rounded-md mt-4 hover:bg-primary-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    >
        {isSubmitting ? 'Posting...' : 'Post'}
    </button>
</form>
