# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Fixed

### Removed

### Security

---

## [1.1.1] - 2026-06-26

### Added
- Unread/read notification tracking with `read_at` field
- `mark_notification_read` command for marking individual notifications as read
- `mark_all_notifications_read` command for bulk read marking
- Live notification read-state synchronization via `notification://read` event
- Frontend badge now displays unread count instead of total count
- Mark-all-read trigger after notification panel is open for 400ms (cancellable on close)
- Visual unread indicator on notification cards (blue left border, bold title)

### Changed
- Updated README with badges, project status, roadmap checklists, and refined sections
- Improved download section to recommend the official website

### Fixed
- Fixed stale badge count when notifications were marked as read externally
- Fixed notification badge to update on `notification://read` events

### Security

- None.

---

## [1.1.0] - 2026-06-25

### Added
- `timer_completed` command for focus timer completion notifications
- Focus timer popup triggers backend notification on natural completion
- Centralized `notify_and_store` helper with persistence and live sync
- Live notification list synchronization via `notification://added` event
- Notification deletion from frontend with immediate backend sync
- Clear All confirmation dialog with loading state
- `closeFloatingPanels()` helper for mutually exclusive floating overlays
- Notification panel closes update banner automatically and vice versa
- LICENSE file (MIT)

### Changed
- Migrated all runtime notification sources to use `notify_and_store`
- Notification badge now reflects unread count

### Fixed
- Fixed duplicate notification on timer completion with `completionNotified` guard

### Security

- None.

---

## [1.0.0] - 2026-06-24

### Added
- Screen time tracking with per-application breakdown
- Daily dashboard with live tracking status
- 7-day analytics with historical archive and daily rollover
- Focus timer with configurable durations (15 / 25 / 45 minutes or custom)
- Floating focus timer popup window
- Timer completion notifications
- Persistent notification center with live synchronization
- Delete individual notifications
- Clear all notifications
- Breathing exercise with guided inhale/exhale phases
- Calm background music with volume control
- Settings page with tracking, notification, and close-behavior toggles
- System tray integration with quick access
- Launch on startup support
- Automatic update checker against GitHub Releases
- Privacy-first local JSON storage (`touchgrass_state.json`)

### Changed

- Initial public release of TouchGrass.

### Security

- None.

---

Future releases will continue to follow the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).