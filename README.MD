# **Syncrab** ✨

A Rust **TUI** and **CLI** for automating your backup jobs. [![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

![Demo Video](assets/demo.gif)

---

## **📦 Features**

- **Three Backup Types**

  - **Real-time**: Watch folders and sync changes instantly
  - **Daily**: Run backups once per day at a specified hour
  - **Weekly**: Run backups once per week on a specified day and hour

- **Backup Modes**

  - **Mirror**: Destructive sync - deletes files in target if deleted in source
  - **Additive**: Copy new/modified files without deleting anything in target

- **Management Tools**
  - Interactive TUI with mouse support
  - Search and filter jobs
  - Clone jobs
  - Bulk replace source/target paths
  - Toggle jobs active/inactive
  - View backup logs with success/failure tracking

---

## **🔧 Pre-Requisites**

- **Linux** (x86_64 or aarch64), **macOS** (Intel or Apple Silicon), or **Windows 10+** (x64).
- No Rust toolchain, no SQLite install, no compiler — the release binaries are self-contained.

Only building from source needs Rust 1.85+ (edition 2024).

---

## **🚀 Usage**

1. **Install** (also how you update):

   **Linux / macOS**

   ```bash
   curl -fsSL https://raw.githubusercontent.com/O-ELMA/syncrab/master/install.sh | sh
   ```

   **Windows** (PowerShell)

   ```powershell
   irm https://raw.githubusercontent.com/O-ELMA/syncrab/master/install.ps1 | iex
   ```

   The installer picks the release for your OS and CPU, verifies its SHA-256 checksum, installs `syncrab`, `syncrab_b` and `syncrab_w` into `~/.local/bin` (`%LOCALAPPDATA%\syncrab` on Windows), and wires up the background jobs:

   | | Watcher (`syncrab_w`) | Hourly backup (`syncrab_b`) |
   |---|---|---|
   | Linux | systemd user service | cron |
   | macOS | launchd agent | launchd agent |
   | Windows | Task Scheduler, at logon | Task Scheduler, hourly |

   Jobs and logs live in:

   - Linux: `~/.local/share/syncrab/syncrab.db` (honours `$XDG_DATA_HOME`)
   - macOS: `~/Library/Application Support/syncrab/syncrab.db`
   - Windows: `%LOCALAPPDATA%\syncrab\syncrab.db`

2. **Run the TUI**:

   ```bash
   syncrab
   ```

3. **Run the backup job manually** (whenever you want):

   - `syncrab_b` accepts an optional frequency argument (`all`, `realtime`, `daily`, `weekly`), followed by an optional status (`active`, `inactive`):

     ```bash
     syncrab_b                  # sync active daily & weekly jobs that match the current hour and day
     syncrab_b all              # sync all jobs (active and inactive)
     syncrab_b realtime         # sync all realtime jobs (active and inactive)
     syncrab_b daily active     # sync only active daily jobs
     syncrab_b weekly inactive  # sync only inactive weekly jobs
     ```

4. **Uninstall**:

   ```bash
   curl -fsSL https://raw.githubusercontent.com/O-ELMA/syncrab/master/uninstall | sh
   ```

   ```powershell
   irm https://raw.githubusercontent.com/O-ELMA/syncrab/master/uninstall.ps1 | iex
   ```

   This removes the binaries, the background jobs and the database.

5. **Build from source** (development):

   ```bash
   git clone https://github.com/O-ELMA/syncrab.git && cd syncrab
   cargo build --release --bins
   ./target/release/syncrab
   ```

---

## **🗝️ Keymaps**

You can use mouse clicks & scrolls across the majority of the app componets.

### **Field Keymaps:**

- `Left/Right arrows`: Move cursor
- `Up/Down (or mouse scroll)`: Navigate fields
- `Backspace & Del`: Delete
- `Ctrl+W`: Delete previous word
- `Ctrl+V`: Paste
- `Enter`: Save

### **Table Keymaps:**

- `Up/Down (or mouse scroll)`: Scroll through jobs & logs
- `Enter`: Open a job or log
- `N`: Create new job
- `Del`: Delete selected job
- `Ctrl+C`: Clone selected job
- `M`: Toggle backup job mirror mode
- `Space`: Toggle backup job activation
- `Alt+Space`: Disable visible backup jobs
- `Ctrl+Space`: Enable visible backup jobs

### **General Keymaps:**

- `f`: Cycle filters
- `s`: Jump to search
- `r`: Jump to realtime table
- `d`: Jump to daily table
- `w`: Jump to weekly table
- `q`: Quit app
- `Tab`: Toggle between logs/jobs menus
- `Ctrl+r`: Replace text in visible jobs' source/target fields

---

## **📝 Inputs**

### **Source**

- **Type**: Text
- **Description**: The absolute/complete path of the file/folder that you want to copy somewhere else.

### **Target**

- **Type**: Text
- **Description**: The absolute/complete path of the destination folder where the source file/folder will be copied to.

### **Hour**

- **Type**: 2 digits (00-23)
- **Description**: The hour (24-hour format) when the job should run. This can be any number from 00 to 23.

### **Day**

- **Type**: 3 characters (Mon, Tue, Wed, Thu, Fri, Sat, Sun)
- **Description**: Specifies the day when the job should run. Use the three-letter abbreviation for the day (e.g., Mon for Monday, Tue for Tuesday...).

---

## **🎮 Usage Example & Default Behavior**

Suppose you enter the following paths:

- **📂 Source:** `/home/my files/My Pictures`
- **📂 Target:** `~/my hard drive/PC/Backed up Pictures`

### I. as a **Daily** or **Weekly** backup:

1. When the job runs, the **`My Pictures`** folder is copied **as a folder**, including all files and subfolders to the target folder.
   The resulting structure will be:

   ```
   ~/my hard drive/PC/Backed up Pictures/My Pictures
   ```

2. Non-existent folders (e.g., `my_hard_drive`, `PC`, `Backed_up_Pictures`) are automatically created if not already created.

3.1. If the `Mirror` field is set to ✅: the copying process mirorrs **destructively** the source folder to the target folder.
New/modified files are added in the target.
If a file is deleted in the source, it’s deleted in the target on the next backup.

3.2. If the `Mirror` field is set to ❌: the copying process will copy **additively** the source folder to the target folder.
New/modified files are added without deleting any files in the target.

### II. as a **Real-time** backup:

1. A watcher monitors **`My Pictures`** folder, its files and subfolders, and mirrors new/modified/deleted files to the target.

   - The **`My Pictures`** folder itself is not copied into the target.

   - Only the items inside **`My Pictures`** are mirrored (added, updated, or deleted) in the target folder.

2. For example, if you add a new file **`image.png`** inside **`My Pictures`**, it will be copied directly into the target directory, resulting in:

   ```
   ~/my hard drive/PC/Backed up Pictures/image.png
   ```

   (Notice that the **`My Pictures`** folder was not copied in this mode.)

3. The watcher only mirrors NEW changes in the folder, such as added, modified, or deleted files and subfolders, rather than recopying everything. This means any files or folders that were already inside **`My Pictures`** before you added it to real-time backups will not be copied.

### III. Path auto-completion:

When creating/updaing a backup job, path auto-completion is set to help you.

Currently, the only expandable symbol is the `~`. It exapands automatically to your absolute home path: `/home/username/`

Variable resolution will be added in the future.

---

## License

This project is licensed under the [MIT License](LICENSE).
