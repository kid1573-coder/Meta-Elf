# Project Guidelines

## Code Style
Follow UI consistency guidelines in [docs/UI_CONSISTENCY.md](docs/UI_CONSISTENCY.md) for fonts, colors, form controls, and modals. Use semantic CSS variables from `src/App.vue` instead of hardcoded colors. Maintain font size hierarchy with `em` units relative to `.yj-shell` for scalability.

## Architecture
This is a Vue 3 + Tauri 2 desktop application for stock watching. Frontend uses Vue Router for navigation, with composables for shared logic. Backend (Rust) handles system tray, window management, and quote fetching from East Money APIs. Components are organized in `src/components/`, views in `src/views/`, and utilities in `src/utils/`.

Key boundaries: UI components in Vue, system integration in Tauri/Rust, data fetching and processing in Rust with unified `QuoteRow` interface.

## Build and Test
- Install: `npm install`
- Dev: `npm run dev` (Vite on port 1430) or `npm run tauri dev` (full app)
- Build: `npm run build` then `npm run tauri build`
- Test: No automated tests configured yet

## Conventions
- Use `yj-field-control` class for all form inputs/selects
- Quotes data from East Money HTTP APIs (may have rate limits)
- Settings stored in `%APPDATA%\yuanjingling\settings.json`
- Window transparency and overlay behavior for floating desktop experience
- Avoid `rem` units inside `#yj-root`; use `em` for theme-scaling text