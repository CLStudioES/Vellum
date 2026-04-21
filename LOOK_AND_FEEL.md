# Vellum — Look & Feel

## 1. Typography (Fontsource)

- **Interface:** Geist Sans.
- **Data:** IBM Plex Mono.
- **Branding:** Source Serif 4.

## 2. Color Palette

- **Background:** Slate-950 (#020617).
- **Borders:** Slate-800 (0.5px).
- **Primary Accent:** Indigo-500.
- **Status:** Emerald-400 (Success), Rose-400 (Error), Amber-400 (Warning).
- **Auth Accent:** Indigo-400 for login/register active states.
- **Connection:** Emerald-500 dot (connected), Amber-500 dot (syncing), Rose-500 dot (offline/error).

## 3. Design Rules

- **Borders:** rounded-md (6px) for containers, rounded-sm (2px) for inputs/tags.
- **Density:** Use tracking-tight in UI and 13px monospaced fonts for data.
- **Transitions:** 150ms-200ms on hover states.
- **Auth Screens:** Centered card layout, max-w-sm, same dark background. No separate theme.
- **Loading States:** Subtle pulse animation on action buttons during API calls. No full-screen spinners.

## 4. Data Handling

- **Privacy:** backdrop-blur on sensitive values by default.
- **Casing:** Variable names always in uppercase with monospaced font.
- **Role Indicators:** Subtle badge next to project name — `owner` (indigo), `editor` (emerald), `viewer` (slate).
- **Viewer Restrictions:** Blur enforced, no "Show" toggle available. Read-only state across all inputs.
- **E2E Indicator:** Small lock icon next to encrypted entries to signal client-side encryption status.
