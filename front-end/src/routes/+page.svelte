<svelte:head>
  <title>LittyPicky - Clean the World</title>
</svelte:head>

<div class="relative bg-slate-900">
  <img src="/dartmoor-1.webp" alt="Dartmoor landscape" class="absolute inset-0 w-full h-full object-cover" loading="lazy" decoding="async">
  <div class="absolute inset-0 bg-slate-900/40"></div>
  <div class="relative max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="py-12 sm:py-16 lg:py-20">
      <div class="max-w-2xl bg-white/85 rounded-2xl backdrop-blur border border-white/40 shadow-xl p-5 sm:p-8">
        <p class="text-sm font-semibold tracking-wide text-primary-700 uppercase">Community cleanup platform</p>
        <h1 class="mt-3 text-3xl tracking-tight font-extrabold text-slate-900 sm:text-4xl md:text-5xl">
          <span class="block xl:inline">Clean the world</span>
          <span class="block text-primary-600 xl:inline">one spot at a time.</span>
        </h1>
        <p class="mt-3 text-sm text-slate-600 sm:text-base">
          Join the global community of litter pickers. Report messy spots, clean them up, earn points, and make your neighborhood shine.
        </p>
        <div class="mt-6 sm:flex sm:justify-start gap-3">
          <a href="/auth/register" class="inline-flex items-center justify-center px-6 py-2.5 border border-transparent text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 sm:text-base">
            Get Started
          </a>
          <a href="/app/feed" class="inline-flex items-center justify-center px-6 py-2.5 border border-primary-200 text-sm font-medium rounded-md text-primary-700 bg-white hover:bg-primary-50 sm:text-base">
            View Feed
          </a>
        </div>
        <div class="mt-8 grid grid-cols-3 gap-3 text-center sm:text-left">
          <div class="rounded-lg bg-white/80 border border-slate-200 p-3">
            <p class="text-xl font-bold text-slate-900">Report</p>
            <p class="text-xs text-slate-500">in seconds</p>
          </div>
          <div class="rounded-lg bg-white/80 border border-slate-200 p-3">
            <p class="text-xl font-bold text-slate-900">Verify</p>
            <p class="text-xs text-slate-500">with trust</p>
          </div>
          <div class="rounded-lg bg-white/80 border border-slate-200 p-3">
            <p class="text-xl font-bold text-slate-900">Earn</p>
            <p class="text-xs text-slate-500">community points</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="py-10 bg-white">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="lg:text-center">
      <h2 class="text-base text-primary-600 font-semibold tracking-wide uppercase">How it works</h2>
      <p class="mt-2 text-3xl leading-8 font-extrabold tracking-tight text-slate-900 sm:text-4xl">
        Simple, Fast, Impactful
      </p>
      <p class="mt-4 max-w-2xl text-slate-600 mx-auto">
        Built for local action and global momentum. Follow a simple flow that keeps cleanups visible and verified.
      </p>
    </div>

    <div class="mt-12 grid gap-6 md:grid-cols-3">
      <div class="rounded-xl border border-slate-200 bg-slate-50 p-6">
        <div class="flex items-center justify-center h-12 w-12 rounded-md bg-primary-600 text-white text-2xl">📸</div>
        <h3 class="mt-4 text-lg font-semibold text-slate-900">Report litter</h3>
        <p class="mt-2 text-sm text-slate-600">Snap a photo and tag the location. Your report instantly appears on the community feed.</p>
      </div>
      <div class="rounded-xl border border-slate-200 bg-slate-50 p-6">
        <div class="flex items-center justify-center h-12 w-12 rounded-md bg-primary-600 text-white text-2xl">🧤</div>
        <h3 class="mt-4 text-lg font-semibold text-slate-900">Pick it up</h3>
        <p class="mt-2 text-sm text-slate-600">Claim a report, clean the spot, and upload an after photo to show the result.</p>
      </div>
      <div class="rounded-xl border border-slate-200 bg-slate-50 p-6">
        <div class="flex items-center justify-center h-12 w-12 rounded-md bg-primary-600 text-white text-2xl">🏆</div>
        <h3 class="mt-4 text-lg font-semibold text-slate-900">Get verified</h3>
        <p class="mt-2 text-sm text-slate-600">Trusted verifiers confirm cleanups so you earn points and climb the leaderboard.</p>
      </div>
    </div>
  </div>
</div>
