<script setup lang="ts">
const version = '0.1.0'
const releasesUrl = 'https://github.com/markbrutx/Tabby/releases'
const appleArmUrl = `${releasesUrl}/download/v${version}/Tabby_${version}_aarch64.dmg`
const appleIntelUrl = `${releasesUrl}/download/v${version}/Tabby_${version}_x64.dmg`

const cards = [
  {
    arch: 'Apple Silicon',
    chip: 'M1 / M2 / M3 / M4',
    url: appleArmUrl,
    file: `Tabby_${version}_aarch64.dmg`,
    primary: true,
  },
  {
    arch: 'Intel x64',
    chip: 'Core i5 / i7 / i9',
    url: appleIntelUrl,
    file: `Tabby_${version}_x64.dmg`,
    primary: false,
  },
]
</script>

<template>
  <section id="download" class="download-section">
    <div class="download-container">
      <div class="download-header">
        <span class="download-version">v{{ version }}</span>
        <h2 class="download-title">Download Tabby</h2>
        <p class="download-subtitle">macOS 13+ required. Universal binary coming soon.</p>
      </div>

      <div class="download-grid">
        <a
          v-for="card in cards"
          :key="card.arch"
          :href="card.url"
          class="download-card"
          :class="{ 'download-card--primary': card.primary }"
        >
          <div class="download-card-icon">
            <svg width="36" height="36" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12.63 2.05c.8-1.05 2.1-1.6 3.12-1.58.13 1.25-.43 2.6-1.15 3.5-.66.86-1.94 1.5-2.95 1.39-.18-1.25.4-2.45.98-3.31zm3.76 3.82c-1.37-.02-2.58.85-3.3.85-.75 0-1.74-.75-2.88-.73-1.46.03-2.82.85-3.58 2.18-1.54 2.68-.4 6.64 1.1 8.81.74 1.05 1.6 2.22 2.74 2.18 1.1-.05 1.54-.7 2.87-.7 1.34 0 1.76.7 2.9.68 1.16-.02 1.93-1.08 2.65-2.13.83-1.21 1.17-2.38 1.2-2.44-.03-.02-2.3-1.02-2.33-3.4-.04-1.98 1.6-2.92 1.68-2.97-1-1.46-2.58-1.6-3.13-1.63z" />
            </svg>
          </div>
          <div class="download-card-body">
            <h3 class="download-card-arch">{{ card.arch }}</h3>
            <span class="download-card-chip">{{ card.chip }}</span>
          </div>
          <div class="download-card-action">
            <span class="download-button">Download .dmg</span>
            <code class="download-card-file">{{ card.file }}</code>
          </div>
        </a>
      </div>

      <div class="download-footer">
        <p class="download-note">
          <strong>Unsigned binary</strong> — on first launch, right-click the app and choose
          <em>Open</em> to bypass Gatekeeper.
        </p>
        <a :href="releasesUrl" class="download-all-releases">
          View all releases →
        </a>
      </div>
    </div>
  </section>
</template>

<style scoped>
.download-section {
  max-width: 800px;
  margin: 0 auto;
  padding: 64px 24px;
}

.download-container {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  border-radius: 24px;
  padding: 56px 48px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.04);
}

.dark .download-container {
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.2);
}

.download-header {
  text-align: center;
  margin-bottom: 48px;
}

.download-version {
  display: inline-block;
  font-family: var(--vp-font-family-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
  padding: 4px 12px;
  border-radius: 20px;
  margin-bottom: 16px;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.download-title {
  font-size: 36px;
  font-weight: 700;
  color: var(--vp-c-text-1);
  margin: 0 0 12px;
  letter-spacing: -0.03em;
}

.download-subtitle {
  font-size: 16px;
  color: var(--vp-c-text-2);
  margin: 0;
}

.download-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-bottom: 40px;
}

@media (max-width: 640px) {
  .download-grid {
    grid-template-columns: 1fr;
  }

  .download-container {
    padding: 40px 24px;
  }
}

.download-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 20px;
  padding: 36px 24px;
  border-radius: 16px;
  border: 1px solid var(--vp-c-border);
  background: var(--vp-c-bg);
  text-decoration: none !important;
  color: inherit;
  transition: all 0.2s ease;
}

.download-card:hover {
  border-color: var(--vp-c-brand-1);
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
}

.dark .download-card:hover {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
}

.download-card--primary {
  border-color: var(--vp-c-brand-1);
  position: relative;
}

.download-card--primary::before {
  content: 'Recommended';
  position: absolute;
  top: -12px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--vp-c-brand-1);
  color: #120b08; /* Dark brown for maximum contrast against brand color */
  font-size: 11px;
  font-weight: 700;
  padding: 4px 12px;
  border-radius: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.download-card-icon {
  color: var(--vp-c-text-1);
  opacity: 0.8;
  margin-bottom: 4px;
  transition: all 0.2s ease;
}

.download-card--primary .download-card-icon {
  color: var(--vp-c-brand-1);
  opacity: 1;
}

.download-card-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-grow: 1;
}

.download-card-arch {
  font-size: 22px;
  font-weight: 700;
  color: var(--vp-c-text-1);
  margin: 0;
}

.download-card-chip {
  font-size: 14px;
  color: var(--vp-c-text-2);
}

.download-card-action {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
  margin-top: 8px;
}

.download-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 12px 0;
  font-size: 15px;
  font-weight: 600;
  border-radius: 8px;
  background: var(--vp-c-bg-mute);
  color: var(--vp-c-text-1);
  transition: all 0.2s ease;
}

.download-card--primary .download-button {
  background: var(--vp-c-brand-1);
  color: #120b08; /* Dark text on brand background for contrast */
}

.download-card:hover .download-button {
  background: var(--vp-c-brand-1);
  color: #120b08;
}

.download-card-file {
  font-family: var(--vp-font-family-mono);
  font-size: 12px;
  color: var(--vp-c-text-3);
  opacity: 0.8;
}

.download-footer {
  text-align: center;
  padding-top: 32px;
  border-top: 1px solid var(--vp-c-border);
}

.download-note {
  font-size: 14px;
  color: var(--vp-c-text-2);
  margin: 0 0 16px;
  line-height: 1.6;
}

.download-note strong {
  color: var(--vp-c-text-1);
  font-weight: 600;
}

.download-all-releases {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 15px;
  font-weight: 600;
  color: var(--vp-c-brand-1);
  text-decoration: none !important;
  transition: opacity 0.2s;
}

.download-all-releases:hover {
  opacity: 0.8;
}
</style>