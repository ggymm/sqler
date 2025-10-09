# Execution Plan

1. **Connection Data Model & Cache** ✅
   - Extend `ConnectionConfig` to capture full connection settings for all supported database types.
   - Ensure cache serialization/deserialization persists complete connection state (host, credentials, etc.).

2. **Driver Integration & Connection Lifecycle**
   - Translate `ConnectionConfig` into driver `ConnectionParams`.
   - Implement activation flow (open/test connection, track status).

3. **Sidebar Interactions**
   - Add double-click to connect, right-click context menu (view/edit/delete), and selection handling.
   - Surface connection status badges and update persistence hooks.

4. **Connection Dialog Enhancements**
   - Hook view/edit actions into existing dialog; pre-fill form from cached config.
   - Add connection info modal with live status messages.

5. **Main Content Tabs**
   - Introduce per-database tab components (starting with MySQL) for Tables/Queries/Functions/Users.
   - Wire topbar tab selection to render the appropriate component.

6. **UI Polish & Testing**
   - Verify responsive behavior (modal sizing, scrolling).
   - Run end-to-end regression (format, `cargo check`) and update documentation/screenshots as needed.
