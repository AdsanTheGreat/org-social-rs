# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to (as crates are supposed to) [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Update to facilitate org-social-lib-rs 0.2.0 update

### Added
- **Text formatting**: Introduced support for underline and strikethrough text
- **Notification View Mode**: Introduced a new view mode in the TUI to display notifications
  - Based on the normal feed display
  - Shows mentions and replies targeted at the user
  - Accessible by pressing 't' to cycle through List -> Threaded -> Notifications view modes
  - Displays notification type indicators ([MENTION], [REPLY], [MENTION+REPLY])
  - Provides a dedicated interface to see only the most important posts for the user

### Changed
- **Major TUI Content Parsing Rewrite**: Completely rewrote the content parsing system to use org-social-lib-rs's post auto-tokenization
  - Replaced line-by-line content processing with token-by-token processing using `Post::tokens()` and `Post::blocks()` methods
  - Simplified content rendering logic by leveraging pre-parsed tokens instead of manual parsing
- Updated view mode cycling to include notifications: List -> Threaded -> Notifications -> List
- Enhanced navigation system to support notification feed navigation
- Updated UI components to handle notification display alongside existing post lists
- Help view is now scrollable, and contains up to date info
- - **Enhanced Mention Support**: Improved mention handling in activatable elements
  - Mentions are now displayed with distinct styling when compared to links
  - Added dedicated mention tracking and focus handling in the activatable system
  - Mentions have the correct url now, that should lead straight to the social.org file

### Technical Details
- **Notification mode**:
  - Added `ViewMode::Notifications` enum variant
  - Extended `TUI` struct with `notification_feed` field
  - Updated navigation methods to handle notification feed
  - Modified UI rendering pipeline to support notification view
  - Added notification-specific post list rendering function
- **Mention System Enhancements**:
  - Added `ActivatableType::Mention` variant to distinguish mentions from regular links
  - Implemented `add_mention()` method in `ActivatableManager`
  - Added `is_mention_focused()` helper method for focus checking
  - Created `create_mention_span()` styling function with cyan color scheme
  - Added `collect_mention()` function for proper mention collection during rendering
  - Updated token processing in content.rs to use mention-specific functions


## [0.1.1] - 27-08-2024
- Fixed some issues with posts creation
## [0.1.0] - 27-08-2024
- Initial CLI and TUI implementation
- Feed and threaded view support
- Post and reply system
- Basic org-social integration
- Supports org-social-lib-rs 0.1.0
