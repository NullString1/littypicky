<script lang="ts">
  // Landing Page
</script>

<svelte:head>
  <title>LittyPicky - Clean the World</title>
</svelte:head>

<div class="relative overflow-hidden bg-white">
  <!-- Hero Section -->
  <div class="max-w-7xl mx-auto">
    <div class="relative z-10 pb-8 bg-white sm:pb-16 md:pb-20 lg:max-w-2xl lg:w-full lg:pb-28 xl:pb-32">
      <main class="mt-10 mx-auto max-w-7xl px-4 sm:mt-12 sm:px-6 md:mt-16 lg:mt-20 lg:px-8 xl:mt-28">
        <div class="sm:text-center lg:text-left">
          <h1 class="text-4xl tracking-tight font-extrabold text-slate-900 sm:text-5xl md:text-6xl">
            <span class="block xl:inline">Clean the world</span>
            <span class="block text-primary-600 xl:inline">one spot at a time.</span>
          </h1>
          <p class="mt-3 text-base text-slate-500 sm:mt-5 sm:text-lg sm:max-w-xl sm:mx-auto md:mt-5 md:text-xl lg:mx-0">
            Join the global community of litter pickers. Report messy spots, clean them up, earn points, and make your neighborhood shine.
          </p>
          <div class="mt-5 sm:mt-8 sm:flex sm:justify-center lg:justify-start">
            <div class="rounded-md shadow">
              <a href="/auth/register" class="w-full flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 md:py-4 md:text-lg md:px-10">
                Get Started
              </a>
            </div>
            <div class="mt-3 sm:mt-0 sm:ml-3">
              <a href="/app/feed" class="w-full flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-primary-700 bg-primary-100 hover:bg-primary-200 md:py-4 md:text-lg md:px-10">
                View Feed
              </a>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
  
  <!-- Hero Image -->
  <div class="lg:absolute lg:inset-y-0 lg:right-0 lg:w-1/2">
    <div class="h-56 w-full bg-primary-100 sm:h-72 md:h-96 lg:w-full lg:h-full flex items-center justify-center">
       <!-- Placeholder for Hero Image -->
       <div class="text-center p-8">
          <span class="text-6xl mb-4 block">🌍</span>
          <p class="text-primary-800 font-bold text-lg">Every Piece Counts</p>
       </div>
    </div>
  </div>
</div>

<!-- Features Section -->
<div class="py-12 bg-slate-50">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="lg:text-center">
      <h2 class="text-base text-primary-600 font-semibold tracking-wide uppercase">How it works</h2>
      <p class="mt-2 text-3xl leading-8 font-extrabold tracking-tight text-slate-900 sm:text-4xl">
        Simple, Fast, Impactful
      </p>
    </div>

    <div class="mt-10">
      <dl class="space-y-10 md:space-y-0 md:grid md:grid-cols-3 md:gap-x-8 md:gap-y-10">
        <!-- Feature 1 -->
        <div class="relative">
          <dt>
            <div class="absolute flex items-center justify-center h-12 w-12 rounded-md bg-primary-500 text-white text-2xl">
              📸
            </div>
            <p class="ml-16 text-lg leading-6 font-medium text-slate-900">Report Litter</p>
          </dt>
          <dd class="mt-2 ml-16 text-base text-slate-500">
            Snap a photo, tag the location. Your report instantly appears on the global feed for pickers to find.
          </dd>
        </div>

        <!-- Feature 2 -->
        <div class="relative">
          <dt>
            <div class="absolute flex items-center justify-center h-12 w-12 rounded-md bg-primary-500 text-white text-2xl">
              🧤
            </div>
            <p class="ml-16 text-lg leading-6 font-medium text-slate-900">Pick It Up</p>
          </dt>
          <dd class="mt-2 ml-16 text-base text-slate-500">
            See a report nearby? Go there, clean it up, and upload a "cleaned" photo to claim your victory.
          </dd>
        </div>

        <!-- Feature 3 -->
        <div class="relative">
          <dt>
            <div class="absolute flex items-center justify-center h-12 w-12 rounded-md bg-primary-500 text-white text-2xl">
              🏆
            </div>
            <p class="ml-16 text-lg leading-6 font-medium text-slate-900">Get Verified</p>
          </dt>
          <dd class="mt-2 ml-16 text-base text-slate-500">
            Trusted verifiers confirm your cleanup. You earn points, badges, and climb the leaderboard.
          </dd>
        </div>
      </dl>
    </div>
  </div>
</div>
