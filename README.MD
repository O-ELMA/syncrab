# **Syncrab** ✨

A Rust **TUI** and **CLI** for automating your backup jobs. [![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

![Demo Video](assets/demo.gif)

---

## **🔧 Pre-Requisites**

1. **Rust**: Install `rustup` to compile and run. [(more on Rust)](https://www.rust-lang.org/tools/install).
2. **SQLite**: Install `sqlite` to store backup jobs & logs. [(more on SQLite)](https://sqlite.org/).

   For arch-based systems, you can use `yay`:
   ```bash 
   yay -S rustup sqlite --noconfirm && rustup default stable
   ```

   or `paru`:
   ```bash 
   paru -S rustup sqlite --noconfirm && rustup default stable
   ```

---

## **🚀 How to Use**

1. **Install**:

   ```bash
   git clone https://github.com/O-ELMA/syncrab.git && cd syncrab && bash ./install
   ```
   or update:
   ```bash
   git clone https://github.com/O-ELMA/syncrab.git && cd syncrab && bash ./update
   ```


2. **Run the TUI**:

   ```bash
   syncrab
   ```

3. **Run the backup job manually** (whenever you want):

   * `syncrab_b` accepts an optional frequency argument (`all`, `realtime`, `daily`, `weekly`), followed by an optional status (`active`, `inactive`):

     ```bash
     syncrab_b                  # sync active daily & weekly jobs that match the current hour and day
     syncrab_b all              # sync all jobs (active and inactive)
     syncrab_b realtime         # sync all realtime jobs (active and inactive)
     syncrab_b daily active     # sync only active daily jobs
     syncrab_b weekly inactive  # sync only inactive weekly jobs
     ```
4. **Uninstall**:

   Run the uninstall script:

   ```bash
   bash ./uninstall
   ```

---

## **🗝️ Keymaps**

You can use mouse clicks & scrolls across the majority of the app componets.

### **Field Keymaps:**
- `Left/Right arrows`: Move the cursor
- `Up/Down (or mouse scroll)`: Navigate between fields
- `Backspace & Del`: Delete characters
- `Ctrl+W`: Delete previous word
- `Ctrl+V`: Paste
- `Enter`: Save job

### **Table Keymaps:**
- `Up/Down (or mouse scroll)`: Scroll through jobs
- `Enter`: Open a job or log record
- `N`: Create new job
- `Del`: Delete selected job
- `Ctrl+C`: Clone selected job
- `M`: Toggle backup job mirroring (Explained in the Bheavior section below)
- `Space`: Toggle backup job activation
- `Alt+Space`: Disable visible backup jobs
- `Ctrl+Space`: Enable visible backup jobs

### **General Keymaps:**
- `f`: Next job filter
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

* **Type**: Text
* **Description**: The absolute/complete path of the file/folder that you want to copy somewhere else.

### **Target**

* **Type**: Text
* **Description**: The absolute/complete path of the destination folder where the source file/folder will be copied to.

### **Hour**

* **Type**: 2 digits (00-23)
* **Description**: The hour (24-hour format) when the job should run. This can be any number from 00 to 23.

### **Day**

* **Type**: 3 characters (Mon, Tue, Wed, Thu, Fri, Sat, Sun)
* **Description**: Specifies the day when the job should run. Use the three-letter abbreviation for the day (e.g., Mon for Monday, Tue for Tuesday...).

---

## **🎮 Usage Example & Default Behavior**

Suppose you enter the following paths:

* **📂 Source:** `/home/my files/My Pictures`
* **📂 Target:** `~/my hard drive/PC/Backed up Pictures`

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
   ````

   (Notice that the **`My Pictures`** folder was not copied in this mode.)

3. The watcher only mirrors NEW changes in the folder, such as added, modified, or deleted files and subfolders, rather than recopying everything. This means any files or folders that were already inside **`My Pictures`** before you added it to real-time backups will not be copied.

### III. Path auto-completion:

When creating/updaing a backup job, path auto-completion is set to help you.

Currently, the only expandable symbol is the `~`. It exapands automatically to your absolute home path: `/home/username/`

Variable resolution will be added in the future.

---

## License

This project is licensed under the [MIT License](LICENSE.txt).
