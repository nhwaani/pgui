# PGUI — Product Roadmap

> Inspired by [DB Browser for SQLite (DB4S)](https://github.com/sqlitebrowser/sqlitebrowser) feature set.
> PGUI extends the concept to **PostgreSQL & MySQL** with a modern GPU-accelerated UI (GPUI).

**Legend:** ✅ Done | 🏗️ In Progress | 📋 Planned | 🔮 Future

---

## 1. 🗄️ Connection & Database Management

### 1.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| Connect to PostgreSQL | ✅ | Host, port, user, password, SSL |
| Connect to MySQL | ✅ | Same connection form |
| SSH Tunneling | ✅ | Keyfile, agent auth, SOCKS proxy |
| Connection persistence (keyring) | ✅ | Password saved to macOS Keychain |
| Switch database at runtime | ✅ | Dropdown in editor toolbar |
| Disconnect | ✅ | Button in editor toolbar |
| Connection status UI | ✅ | Disconnected / Connecting / Connected |
| Database list from server | ✅ | Select widget populated from `pg_catalog` / `SHOW DATABASES` |

### 1.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Recent connections menu** | High | Like DB4S recent files |
| **Connection bookmarks / favourites** | Medium | Named connection profiles |
| **Import connections from other tools** | Medium | DBeaver, DataGrip, etc. |
| **Connection sharing / export** | Low | Share connection config as file |
| **Read-only mode toggle** | Medium | Prevent accidental writes |
| **Multiple simultaneous connections** | High | Tab-per-connection or split views |
| **SSL certificate picker** | Medium | GUI for client cert selection |

---

## 2. 📝 SQL Editor

### 2.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| Multi-line SQL editor | ✅ | Code editor with syntax highlighting |
| LSP completions (tables, columns) | ✅ | Schema-aware autocomplete |
| AI inline completions | ✅ | Powered by Claude via `chat_stateless` |
| AI code actions (Complete/Explain/Optimize) | ✅ | Selected text or full query |
| Format SQL | ✅ | Via `sqlformat` crate |
| Execute query (cursor-aware) | ✅ | Detects multi-query, runs at cursor |
| Toggle AI completions | ✅ | Sparkles button in toolbar |

### 2.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Multiple SQL tabs** | 🔴 **Critical** | Like DB4S multi-tab editor |
| **SQL history / query log** | High | DB4S logs all SQL commands |
| **Find & Replace in editor** | High | DB4S has full FindReplace dialog |
| **Save SQL to file** | High | DB4S can save/load .sql files |
| **Drag-and-drop tables into editor** | Medium | Drop table name from schema tree |
| **SQL block comment toggle** | Medium | Ctrl+Shift+C style |
| **Editor font / theme preferences** | Medium | Preferences dialog |
| **Multi-cursor editing** | Low | Advanced editor feature |
| **Auto-save open SQL files** | Low | Prevent data loss |

---

## 3. 📊 Query Results & Data Browser

### 3.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| Results table (spreadsheet view) | ✅ | Sortable columns, resizable |
| Export to CSV (streaming) | ✅ | Handles large datasets, comma-delimited |
| Export to JSON/NDJSON (streaming) | ✅ | Handles large datasets |
| Execution time / rows affected | ✅ | Displayed in results panel |
| Error display | ✅ | Styled error panel |
| Resizable split between editor/results | ✅ | Vertical resizable panel |

### 3.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Browse full table data (no query)** | 🔴 **Critical** | Like DB4S "Browse Data" tab — click table → see rows |
| **Inline cell editing** | 🔴 **Critical** | Double-click cell → edit value in-place |
| **Add new record** | High | DB4S has AddRecord dialog |
| **Delete record(s)** | High | DB4S row delete with confirmation |
| **Duplicate record** | Medium | DB4S duplicate row feature |
| **Search/filter records** | High | DB4S filter bar per column |
| **Paginated scrolling (load more)** | Medium | Already scaffolded (`load_more_threshold`) |
| **Column display format** | Medium | Format BLOB as hex, date as human, etc. |
| **NULL value styling** | Medium | Already present (`is_null` → italic/grey) |
| **Conditional formatting** | Medium | Highlight cells based on value |
| **Row ID / PK column visibility** | Medium | Toggle `_rowid_` / primary key |
| **Freeze columns** | Low | Keep first N columns fixed |
| **Hide columns** | Low | Per-table column visibility |
| **Column type metadata popup** | Medium | Click cell → see type, nullable, ordinal |
| **Export filtered results** | Medium | Export only visible/filtered subset |
| **Export to TSV** | Medium | Tab-delimited, with configurable delimiter |
| **Export to Excel (XLSX)** | Medium | Beyond CSV/JSON |
| **Export to Parquet** | Low | Apache Parquet columnar format |
| **Export to Markdown** | Low | Pipe-table format for docs |
| **Export to HTML** | Low | Styled HTML table |
| **Import TSV** | Medium | Tab-delimited files → table |
| **Import Excel (XLSX)** | Medium | Read sheets → database table |
| **Import Parquet** | Low | Apache Parquet → table |
| **Copy row/column to clipboard** | Medium | Right-click → copy |

---

## 4. 🏗️ Schema Browser & Table Management

### 4.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| Schema tree (tables & views) | ✅ | Hierarchical sidebar |
| Table column details | ✅ | Click table → show columns as results |
| Schema-aware AI completions | ✅ | Schema sent to LLM for context |

### 4.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Create table dialog** | 🔴 **Critical** | Like DB4S EditTableDialog — column names, types, PK, FK, constraints |
| **Create index** | High | DB4S EditIndexDialog |
| **Modify table (add/drop column)** | High | Alter table via GUI |
| **Delete / drop table** | High | Right-click → drop with confirmation |
| **Delete / drop index** | Medium | Schema tree context menu |
| **Create view** | Medium | Save query as view |
| **Create trigger** | Medium | DB4S trigger editor |
| **Import CSV/TSV into table** | 🔴 **Critical** | DB4S ImportCsvDialog — delimited files → table |
| **Import JSON into table** | Medium | JSON → table |
| **Import Excel (XLSX) into table** | Medium | Read Excel sheets → table |
| **Import Parquet into table** | Low | Apache Parquet format → table |
| **Import from SQL dump** | Medium | DB4S import SQL file |
| **Export table as CSV/TSV** | High | Right-click → export with configurable delimiter |
| **Export table as JSON** | High | Right-click → export |
| **Export table as Excel (XLSX)** | Medium | Right-click → export with sheet name |
| **Export table as Parquet** | Low | Apache Parquet format |
| **Export table as SQL dump** | Medium | DB4S ExportSqlDialog |
| **Export table as Markdown** | Low | Pipe-to-table format for docs |
| **Export table as HTML** | Low | Styled HTML table export |
| **Copy CREATE statement** | Medium | Right-click → copy DDL |
| **Table/column comments** | Low | Read/write SQL comments |
| **Schema comparison** | Low | Diff two databases |

---

## 5. 📈 Data Visualization & Plotting

### 5.1 Current
| Feature | Status | Notes |
|---------|--------|-------|
| None | ❌ | No plotting yet |

### 5.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Basic chart / plot dock** | Medium | DB4S PlotDock — bar, line, scatter from query results |
| **Axis configuration** | Medium | X/Y axis column picker |
| **Multiple Y-axes** | Low | Dual Y-axis support |
| **Legend, color picker** | Low | Styling controls |
| **Export plot as image** | Low | PNG/SVG export |
| **Print plot** | Low | Print dialog |

---

## 6. 🔧 Database Administration

### 6.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| None | ❌ | No admin features yet |

### 6.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Database compact / vacuum** | Medium | DB4S compact action — reclaim space |
| **PRAGMA settings** | Medium | DB4S pragma editor (auto-vacuum, page_size, etc.) |
| **Encryption (SQLCipher)** | Medium | DB4S CipherDialog — encrypt/decrypt |
| **Attach another database** | Medium | DB4S file attach — attach another DB for cross-DB queries |
| **Detach database** | Medium | DB4S file detach |
| **Load extension** | Low | DB4S load extension .so/.dll |
| **Database properties / metadata** | Low | Size, version, row counts |
| **Kill active queries** | Low | Cancel running query (button in status bar) |

---

## 7. 🧩 AI & Productivity

### 7.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| AI inline completions | ✅ | Claude-powered, debounced |
| AI code actions (Complete/Explain/Optimize) | ✅ | User-initiated |
| Agent chat panel | ✅ | Conversational AI assistant |
| Agent tool execution | ✅ | Get schema, tables, columns |
| Schema-aware prompts | ✅ | Context sent to LLM |

### 7.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **AI-powered data analysis** | Medium | "Analyze this table" → summary, outliers |
| **AI query generation from natural language** | Medium | "Show me users who signed up last month" |
| **AI error explanation** | Medium | Explain SQL error in plain English |
| **AI query optimization suggestions** | Medium | Already partially in "Optimize" action |
| **AI data import mapping** | Low | Suggest column types from CSV header |
| **Multi-model support (UI config)** | Medium | Switch model from dropdown |

---

## 8. 🖥️ General UI & UX

### 8.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| GPU-accelerated rendering (Metal) | ✅ | GPUI framework |
| Dark/Light theme | ✅ | Tokyo Night, GitHub themes |
| Resizable panels | ✅ | Vertical split in editor/results |
| Notifications | ✅ | Success/error toasts |
| Header bar | ✅ | App title, connection status |
| Footer bar | ✅ | Toggle sidebar, agent, history |
| History panel | ✅ | Recorded query history |

### 8.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Preferences dialog** | 🔴 **Critical** | DB4S PreferencesDialog — font, theme, SQL settings, extensions |
| **Keyboard shortcuts / keybindings** | High | DB4S has extensive shortcuts |
| **Drag-and-drop file open** | Medium | Drop `.sql` / `.csv` / `.db` files |
| **Status bar improvements** | Medium | Encoding, encryption, row counts, busy indicator |
| **Recent files / projects menu** | High | DB4S recent files with pinned items |
| **Find & Replace in data table** | High | DB4S FindReplaceDialog for cell values |
| **Print table / query results** | Low | DB4S print dialog |
| **Export settings** | Low | Import/export preferences as JSON |
| **Proxy configuration** | Low | DB4S ProxyDialog for network access |
| **Auto-load last project** | Low | DB4S remembers last opened file |
| **Multi-window support** | Low | Detach tabs into separate windows (like DB4S dock tear-off) |
| **Fullscreen mode** | Low | Toggle fullscreen |

---

## 9. 📦 Packaging & Distribution

### 9.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| macOS native app | ✅ | GPUI renders via Metal |
| Homebrew cask | ✅ | Listed in Cargo.toml |

### 9.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Linux support** | High | GPUI supports X11/Wayland |
| **Windows support** | Medium | GPUI supports Windows |
| **App bundle / DMG** | Medium | macOS `.app` bundle |
| **CI/CD pipeline** | Medium | Automated builds + tests |
| **Nightly builds** | Low | DB4S continuous nightly builds |
| **Snap/Flatpak** | Low | Linux packaging |
| **Scoop/Chocolatey** | Low | Windows package managers |
| **Docker image** | Low | For headless / CI use |

---

## 10. 🧪 Testing & Quality

### 10.1 Current (Done)
| Feature | Status | Notes |
|---------|--------|-------|
| Unit tests for connection types | ✅ | 20 tests |
| Unit tests for SSH config | ✅ | 6 tests |
| Unit tests for storage/migration | ✅ | 12 tests |
| Agent builder test | ✅ | 1 test |

### 10.2 Planned
| Feature | Priority | Notes |
|---------|----------|-------|
| **Agent module tests** | High | Tool execution, API calls, message routing |
| **Integration tests** | High | Full query → results pipeline |
| **UI component tests** | Medium | GPUI component testing |
| **Database integration tests** | Medium | Against real PG/MySQL (docker-compose) |
| **Performance benchmarks** | Low | Large dataset rendering |
| **Accessibility testing** | Low | Screen reader, keyboard nav |

---

## 📋 Priority Matrix

| Priority | Features |
|----------|----------|
| 🔴 **Critical** | Multiple SQL tabs, Browse table data, Inline cell editing, Create table dialog, Import CSV/TSV, Preferences dialog |
| 🟠 **High** | Recent connections, Find & Replace (editor + data), Save SQL to file, Search/filter records, Delete records, Add records, Export filtered data (CSV/TSV/JSON), Create index, Keyboard shortcuts, Agent module tests, Integration tests |
| 🟡 **Medium** | Connection bookmarks, Read-only mode, SSL picker, Drag-drop tables, Paginated scrolling, Column display format, Conditional formatting, Null styling, Row ID toggle, Freeze columns, Hide columns, Cell metadata popup, Create view, Export SQL dump, Import SQL dump, Import JSON, Import Excel (XLSX), Export Excel (XLSX), Export TSV, Copy CREATE, Chart/plot dock, Compact/vacuum, PRAGMA settings, Encryption, Attach/detach DB, AI data analysis, AI NL queries, AI error explanation, Multi-model config, Status bar, Recent files, Drag-drop files, Linux support, CI/CD, Windows support |
| 🟢 **Low** | Multi-window, Fullscreen, Print, Export Parquet, Export Markdown, Export HTML, Import Parquet, Export settings, Proxy config, Auto-load project, Schema comparison, Table comments, Multi-cursor, Kill queries, DB properties, Load extensions, Docker, Nightly builds, Snap/Flatpak, Scoop/Chocolatey, App bundle, Performance benchmarks, Accessibility |

---

## 📊 Feature Parity with DB Browser for SQLite (DB4S)

| DB4S Feature | PGUI Status | Target Release |
|--------------|-------------|----------------|
| Open/Save database file | ❌ N/A (server DB) | — |
| Create & compact database files | ❌ | v0.3 |
| Create, define, modify tables | ❌ | v0.3 |
| Create, define, delete indexes | ❌ | v0.3 |
| Browse, edit, add, delete records | ⚡ Partial (browse only) | v0.2 |
| Search records | ❌ | v0.2 |
| Import/export CSV | ⚡ Partial (export only) | v0.2 |
| Import/export TSV | ❌ | v0.2 |
| Import/export Excel (XLSX) | ❌ | v0.3 |
| Import/export Parquet | ❌ | v0.4 |
| Import/export SQL dump | ❌ | v0.3 |
| Import/export JSON | ⚡ Partial (export only) | v0.2 |
| Issue SQL queries & inspect results | ✅ | v0.1 |
| SQL command log | ❌ | v0.2 |
| Plot simple graphs | ❌ | v0.4 |
| Multiple editor tabs | ❌ | v0.2 |
| Find/replace in editor | ❌ | v0.2 |
| Preferences dialog | ❌ | v0.2 |
| Print support | ❌ | v0.4 |
| Encryption (SQLCipher) | ❌ | v0.3 |
| Attach/detach databases | ❌ | v0.3 |
| Extensions | ❌ | v0.4 |
| Proxy configuration | ❌ | v0.4 |

---

## 🚀 Release Plan

> Version milestones grouping features by user-vertical and dependency order.
> Each release should ship a **coherent, testable, valuable increment**.

### v0.1 — "MVP" ✅ *(shipped)*

| Area | What shipped |
|------|-------------|
| 🗄️ Connection | PostgreSQL & MySQL connect, SSH tunnel, keyring, database switch, disconnect |
| 📝 Editor | Multi-line SQL editor, LSP completions, AI inline completions, AI code actions, format, execute, toggle AI |
| 📊 Results | Spreadsheet table, CSV/JSON export, execution time, error display, resizable split |
| 🏗️ Schema | Schema tree, column details, schema-aware AI |
| 🧩 AI | Agent chat, tool execution, schema-aware prompts |
| 🖥️ UI | GPUI Metal render, dark/light theme, resizable panels, notifications, header, footer, history |
| 🧪 Testing | Connection types (20), SSH config (6), storage/migration (12), agent builder (1) |

---

### v0.2 — "Data Browser" 🎯 *(current target)*

**Theme:** Make PGUI a full data browser — browse, edit, search, and import/export data.

| Area | Features | Est. Effort |
|------|----------|-------------|
| 📝 Editor | **Multiple SQL tabs**, SQL history/log, Save SQL to file, Find & Replace | Large |
| 📊 Results | **Browse table data** (click → rows), **Inline cell editing**, Add/delete/duplicate records, Search/filter records, Paginated scrolling, Column display format, NULL styling, Row ID toggle, Copy row/column | Large |
| 📊 Export | **Import CSV/TSV** (dialog), Export TSV, Export filtered subset, Export Markdown, Export HTML | Medium |
| 🖥️ UI | **Preferences dialog**, Keyboard shortcuts, Recent connections menu, Status bar improvements, Recent files/projects | Medium |
| 🧪 Testing | Agent module tests, Integration tests | Small |

**Goal:** Feature parity with DB4S for browsing & editing data.

---

### v0.3 — "Schema Manager"

**Theme:** Full schema lifecycle management — create, alter, drop, document.

| Area | Features | Est. Effort |
|------|----------|-------------|
| 🏗️ Schema | **Create table dialog**, Create index, Modify table (add/drop column), Drop table/index, Create view, Copy CREATE statement, Table/column comments | Large |
| 📦 Import/Export | **Export SQL dump**, **Import SQL dump**, Export Excel (XLSX), Import Excel (XLSX), Import JSON | Medium |
| 🔧 Admin | Compact/vacuum, PRAGMA settings, Encryption (SQLCipher), Attach/detach database | Medium |
| 🧩 AI | AI error explanation, AI data import mapping (CSV → column types) | Small |
| 🖥️ UI | Drag-and-drop tables into editor, SQL block comment toggle, Editor font/theme prefs, Read-only mode toggle, SSL cert picker | Medium |

**Goal:** Feature parity with DB4S for schema management.

---

### v0.4 — "Analytics & Platform"

**Theme:** Data visualization, natural-language queries, cross-platform support.

| Area | Features | Est. Effort |
|------|----------|-------------|
| 📈 Plotting | **Chart/plot dock** (bar, line, scatter), Axis config, Legend/colors, Export plot as image, Print plot | Large |
| 🧩 AI | **AI NL queries** (natural language → SQL), AI data analysis (summaries, outliers), Multi-model config, AI query optimization | Medium |
| 📦 Packaging | **Linux support**, Windows support, App bundle/DMG, CI/CD pipeline | Large |
| 📊 Export | Export Parquet, Import Parquet | Small |
| 🖥️ UI | Multi-window support, Fullscreen mode, Drag-and-drop file open, Connection bookmarks/favourites | Medium |
| 🔧 Admin | Load extensions, Database properties/metadata, Kill active queries, Auto-load last project | Small |

**Goal:** Feature parity with DB4S for analytics & visualization.

---

### v1.0 — "Enterprise Ready" 🏆

**Theme:** Polish, performance, accessibility, distribution.

| Area | Features | Est. Effort |
|------|----------|-------------|
| 📦 Distribution | Nightly builds, Snap/Flatpak, Scoop/Chocolatey, Docker image | Medium |
| 🖥️ UI | Print support, Export settings, Proxy configuration, Schema comparison | Medium |
| 🧪 Quality | Performance benchmarks (large dataset), Accessibility (screen reader, keyboard nav), UI component tests, Database integration tests (docker-compose) | Medium |
| 🧩 Misc | Multi-cursor editing, Connection sharing/export, Import connections from other tools, Auto-save SQL files | Small |

**Goal:** Production-ready, distributable, enterprise-grade SQL client.

---

### 🏁 Beyond v1.0 — "Future" 🔮

| Feature | Notes |
|---------|-------|
| Multiple simultaneous connections | Tab-per-connection or split views |
| Create trigger | DB4S trigger editor |
| Import connections from DBeaver/DataGrip | Cross-tool migration |
| Connection sharing / export | Share config as encrypted file |
| Schema comparison | Diff two databases |
| AI-powered data analysis | Deep-dive analytics |
