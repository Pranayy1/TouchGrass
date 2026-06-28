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

## [1.1.2] - 2026-06-28

### Added

* Automatic unread/read notification tracking using the `read_at` field
* `mark_notification_read` command for individual notifications
* `mark_all_notifications_read` command for bulk read handling
* Live read-state synchronization through the `notification://read` event
* Automatic mark-as-read after the notification panel remains open briefly
* Visual unread indicator (blue accent and bold title)
* Dynamic unread notification badge
* Focus timer completion notifications for both preset and custom timer durations
* Improved popup timer synchronization to preserve the correct session duration across popup reuse

### Changed

* Notification badge now displays unread notification count instead of total notifications
* Improved notification panel rendering reliability
* Improved notification synchronization between backend and frontend
* Updated README with improved badges, project status, roadmap, and download section

### Fixed

* Fixed notification panel rendering when the notification list DOM reference was missing
* Fixed focus timer completion notifications not appearing for custom timer durations
* Fixed popup timer state becoming stale after reopening the popup window
* Fixed stale notification badge after read-state updates
* Fixed several notification synchronization edge cases
* Improved notification persistence reliability

### Security

* None.

---

## [1.1.1] - 2026-06-26

### Added

* Persistent notification center with live synchronization
* Backend notification persistence
* Live notification updates through `notification://added`
* Delete individual notifications
* Clear All notifications
* Automatic 24-hour notification cleanup
* Focus timer completion notification infrastructure
* Centralized `notify_and_store` helper
* Mutually exclusive floating panels via `closeFloatingPanels()`
* MIT License

### Changed

* Migrated runtime notifications to the centralized notification pipeline

### Fixed

* Fixed duplicate timer completion notifications
* Fixed notification synchronization across application windows

### Security

* None.

---

## [1.1.0] - 2026-06-25

### Added

* Screen time tracking with per-application breakdown
* Daily dashboard with live tracking status
* 7-day analytics with historical archive and daily rollover
* Focus timer with configurable durations (15 / 25 / 45 minutes or custom)
* Floating focus timer popup window
* Breathing exercise with guided inhale/exhale phases
* Calm background music with volume control
* Settings page with tracking, notification, and close-behavior toggles
* System tray integration with quick access
* Launch on startup support
* Automatic update checker against GitHub Releases
* Privacy-first local JSON storage (`touchgrass_state.json`)

### Changed

* Initial public release of TouchGrass.

### Security

* None.

---

Future releases will continue to follow the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
