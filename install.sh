#!/bin/sh

# Logging ─────────────────────────────────────────────────────────────────────
log() {
    printf '\033[1;36m💡 [INFO]\033[0m %s\n' "$1"
}
success() {
    printf '\033[1;32m✅ [SUCCESS]\033[0m %s\n' "$1"
}
warning() {
    printf '\033[1;33m⚠️  [WARNING]\033[0m %s\n' "$1"
}
die() {
    printf '\033[1;31m❌ [ERROR]\033[0m %s\n' "$1" >&2
    exit 1
}
line() {
    printf '────────────────────────────────────────────────────────────────────────\n'
}

# Init logic ──────────────────────────────────────────────────────────────────
line
set -eu

REPO="O-ELMA/syncrab"
PREFIX="${PREFIX:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"
LEGACY_PREFIX="$HOME/.cargo/bin"

fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" "$1"
    else
        die "Neither curl nor wget is installed"
    fi
}

sha_check() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c - >/dev/null
    else
        shasum -a 256 -c - >/dev/null
    fi
}

os=$(uname -s)
case "$os" in
    Linux)  suffix="-unknown-linux-musl" ;;
    Darwin) suffix="-apple-darwin" ;;
    *)      die "Unsupported OS: $os (only Linux and macOS)" ;;
esac

arch=$(uname -m)
case "$arch" in
    x86_64|amd64)  target="x86_64$suffix" ;;
    arm64|aarch64) target="aarch64$suffix" ;;
    *)             die "Unsupported architecture: $arch (only x86_64 and aarch64)" ;;
esac

if [ "$VERSION" = latest ]; then
    base="https://github.com/$REPO/releases/latest/download"
else
    base="https://github.com/$REPO/releases/download/v${VERSION#v}"
fi

asset="syncrab-$target.tar.gz"
tmp=""
trap '[ -n "$tmp" ] && rm -rf "$tmp"' EXIT INT TERM
tmp=$(mktemp -d)

log "Downloading syncrab for $target..."
fetch "$base/$asset" "$tmp/$asset"
fetch "$base/SHA256SUMS" "$tmp/SHA256SUMS"

checksum_line=$(grep "  $asset\$" "$tmp/SHA256SUMS" || true)
[ -n "$checksum_line" ] || die "No checksum published for $asset"
printf '%s\n' "$checksum_line" | (cd "$tmp" && sha_check) ||
    die "Checksum mismatch for $asset"

log "Extracting..."
tar -xzf "$tmp/$asset" -C "$tmp"

# Replace binaries in place without tripping over a running executable
if [ "$os" = Linux ] && command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop syncrab_w.service 2>/dev/null || true
else
    launchctl unload "$HOME/Library/LaunchAgents/com.syncrab.watcher.plist" 2>/dev/null || true
fi

log "Installing binaries to $PREFIX..."
mkdir -p "$PREFIX"
for bin in syncrab syncrab_b syncrab_w; do
    install -m 755 "$tmp/syncrab-$target/$bin" "$PREFIX/.$bin.new"
    mv -f "$PREFIX/.$bin.new" "$PREFIX/$bin"
done

if [ "$PREFIX" != "$LEGACY_PREFIX" ] && {
    [ -e "$LEGACY_PREFIX/syncrab" ] ||
    [ -e "$LEGACY_PREFIX/syncrab_b" ] ||
    [ -e "$LEGACY_PREFIX/syncrab_w" ];
}; then
    log "Removing the old cargo installation from $LEGACY_PREFIX..."
    rm -f "$LEGACY_PREFIX/syncrab" "$LEGACY_PREFIX/syncrab_b" "$LEGACY_PREFIX/syncrab_w"
fi

if [ "$os" = Darwin ]; then
    data_dir="$HOME/Library/Application Support/syncrab"
else
    data_dir="${XDG_DATA_HOME:-$HOME/.local/share}/syncrab"
fi

if [ -f "$LEGACY_PREFIX/syncrab.db" ] && [ ! -e "$data_dir/syncrab.db" ]; then
    log "Moving your existing database to $data_dir..."
    mkdir -p "$data_dir"
    mv "$LEGACY_PREFIX/syncrab.db" "$data_dir/syncrab.db"
fi

# Background jobs ─────────────────────────────────────────────────────────────
setup_linux() {
    service_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
    service_file="$service_dir/syncrab_w.service"
    mkdir -p "$service_dir"

    cat > "$service_file" <<EOF
[Unit]
Description=Syncrab Real-Time File Synchroniser

[Service]
ExecStart=$PREFIX/syncrab_w
Restart=on-failure

[Install]
WantedBy=default.target
EOF

    if command -v systemctl >/dev/null 2>&1; then
        if systemctl --user daemon-reload 2>/dev/null &&
            systemctl --user enable --now syncrab_w.service 2>/dev/null; then
            success "Watcher service enabled."
        else
            warning "Could not start the user service (no user session?). Run: systemctl --user enable --now syncrab_w.service"
        fi
    else
        warning "systemd not found; the watcher will not start automatically."
    fi

    cron_line="0 * * * * $PREFIX/syncrab_b --silent"
    if command -v crontab >/dev/null 2>&1; then
        cron_tmp=$(mktemp)
        crontab -l 2>/dev/null | grep -v syncrab_b > "$cron_tmp" || true
        printf '%s\n' "$cron_line" >> "$cron_tmp"
        crontab "$cron_tmp"
        rm -f "$cron_tmp"
        success "Hourly backup scheduled via cron."
    else
        warning "crontab not found. Add this line yourself: $cron_line"
    fi
}

setup_macos() {
    agents="$HOME/Library/LaunchAgents"
    mkdir -p "$agents"

    watcher_plist="$agents/com.syncrab.watcher.plist"
    backup_plist="$agents/com.syncrab.backup.plist"

    cat > "$watcher_plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.syncrab.watcher</string>
    <key>ProgramArguments</key>
    <array>
        <string>$PREFIX/syncrab_w</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/tmp/syncrab_w.err</string>
</dict>
</plist>
EOF

    cat > "$backup_plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.syncrab.backup</string>
    <key>ProgramArguments</key>
    <array>
        <string>$PREFIX/syncrab_b</string>
        <string>--silent</string>
    </array>
    <key>StartInterval</key>
    <integer>3600</integer>
    <key>RunAtLoad</key>
    <false/>
</dict>
</plist>
EOF

    for plist in "$watcher_plist" "$backup_plist"; do
        launchctl unload "$plist" 2>/dev/null || true
        launchctl load -w "$plist" 2>/dev/null ||
            warning "Could not load $(basename "$plist")"
    done

    success "Watcher and hourly backup scheduled via launchd."
    warning "macOS may block the watcher from reading Desktop/Documents. Grant Full Disk Access to $PREFIX/syncrab_w in System Settings → Privacy & Security."
}

if [ "$os" = Linux ]; then
    setup_linux
else
    setup_macos
fi

case ":$PATH:" in
    *":$PREFIX:"*) ;;
    *) warning "$PREFIX is not in your PATH. Add this to your shell profile: export PATH=\"$PREFIX:\$PATH\"" ;;
esac

line
success "Installed syncrab, syncrab_b and syncrab_w in $PREFIX."
log "Run 'syncrab' to open the TUI. Re-run this script any time to update."
