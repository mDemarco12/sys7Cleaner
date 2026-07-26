# sys7 Cleaning Assistant

A disk-cleanup utility for Apple Silicon Macs with a pixel-accurate Macintosh System 7 interface. Built as a Tauri 2 app: a pure-Rust scanning/cleanup engine (`sweep-core`, no GC, no Tauri dependency) driving a hand-written HTML/CSS/JS frontend — no Node, no bundler.

## Project layout

```
crates/sweep-core/    Pure Rust engine: parallel disk sizing, Mach-O arch
                      detection, allowlist-first safety guard, Trash-based
                      reclaim. Fully unit tested, zero Tauri dependency.
crates/sweep-cli/     Headless CLI (scan/plan/apply) over the same engine —
                      useful for testing against a sandboxed --root.
src-tauri/            Tauri shell: commands, capabilities, bundle config.
src/                  Static frontend (no build step): index.html, css/, js/.
```

## Building and running

```bash
cargo tauri dev              # run in development, live window
cargo tauri build             # release build + .app/.dmg bundle
```

The release build produces `src-tauri/target/release/bundle/macos/Macintosh Cleaning Assistant.app`. Copy it to `/Applications` to make it discoverable in Launchpad/Spotlight:

```bash
cp -R "target/release/bundle/macos/Macintosh Cleaning Assistant.app" /Applications/
```

## Code signing (self-signed, local-only)

The app is signed with a **free self-signed certificate** rather than a paid Apple Developer ID. This is enough to:

- Give the app a **stable code signature** across rebuilds, which matters because macOS ties Full Disk Access (and other TCC) grants to the app's designated requirement. Ad-hoc signing (`codesign -s -`, Tauri's default with no identity configured) bakes a hash of that specific build into the requirement, so **every rebuild invalidates the FDA grant** and you have to re-approve it in System Settings each time. Signing with any stable identity — even a self-signed one — fixes this, because the requirement is then based on the certificate's identity, not the binary's hash.

It does **not**:

- Satisfy Gatekeeper for anyone else. A self-signed cert has no trust chain back to Apple, so on any machine other than the one that generated and trusted it, the app still shows the "Apple could not verify this app is free of malware" warning (right-click → Open bypasses it once). Distributing to other people without warnings requires a paid Apple Developer Program membership ($99/yr) plus notarization — see below.

### How the certificate was created

```bash
# 1. Generate a self-signed cert with the Code Signing extended key usage.
openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -days 3650 -nodes \
  -subj "/CN=Mac Cleaning Assistant Dev" \
  -addext "keyUsage=critical,digitalSignature" \
  -addext "extendedKeyUsage=critical,codeSigning" \
  -addext "basicConstraints=critical,CA:FALSE"

# 2. Package as PKCS#12 for import.
openssl pkcs12 -export -out devcert.p12 -inkey key.pem -in cert.pem -passout pass:<a password>

# 3. Import into the login keychain, granting codesign access to the private key
#    without a per-signing password prompt.
security import devcert.p12 -k ~/Library/Keychains/login.keychain-db \
  -P <the password from step 2> -T /usr/bin/codesign -A

# 4. Trust the cert for the codesigning policy ON THIS MACHINE ONLY — this is
#    what makes `security find-identity -v -p codesigning` see it as valid.
security add-trusted-cert -d -r trustRoot -p codeSign \
  -k ~/Library/Keychains/login.keychain-db cert.pem

# 5. Delete the plaintext key/cert/p12 files — the identity now lives in Keychain Access.
rm key.pem cert.pem devcert.p12
```

Verify it's available:

```bash
security find-identity -v -p codesigning
#   1) <fingerprint> "Mac Cleaning Assistant Dev"
#      1 valid identities found
```

### How it's wired into the build

`src-tauri/tauri.conf.json` sets:

```json
"bundle": {
  "macOS": {
    "signingIdentity": "Mac Cleaning Assistant Dev"
  }
}
```

`cargo tauri build` (and `cargo tauri dev`) picks this up automatically and signs with that identity — no manual `codesign` step needed. If you ever need to sign a bundle by hand:

```bash
codesign --force --deep --sign "Mac Cleaning Assistant Dev" --options runtime \
  "/Applications/Macintosh Cleaning Assistant.app"
codesign --verify --deep --strict --verbose=2 "/Applications/Macintosh Cleaning Assistant.app"
```

### Setting this up on a fresh machine / after a keychain wipe

If `security find-identity -v -p codesigning` comes back empty on a machine you're building on, re-run the five steps above to regenerate and re-trust a certificate — the identity is only as portable as the keychain it's imported into. Builds will still succeed without any identity present (Tauri falls back to ad-hoc signing); you'll just be back to the FDA-reset-on-rebuild annoyance described above.


## Branding

The boot splash and a few UI strings were deliberately de-branded from Apple's actual "Welcome to Macintosh" boot screen to avoid using Apple-owned artwork/copy:

- The splash no longer ships a raster screenshot of the real boot screen (which embedded Apple's own small Mac/rainbow-Apple icon). It's now built entirely from HTML/CSS markup — a white bordered dialog box, matching the same visual language as the confirmation alert — with an **original** angular dark-red emblem (`SYS7_EMBLEM_SVG` in `src/js/icons.js`) in place of any OS-vendor logo. That emblem is deliberately its own geometry, not a recreation of any existing insignia (real or fictional) — copying a *different* copyrighted/trademarked logo in place of Apple's wouldn't reduce infringement risk, it would just move it to someone else's IP.
- Splash text: "Welcome to sys7 Cleaner." (was "Welcome to Macintosh.")
- Toolbar button: "Scan" (was "Scan Macintosh")
- Window title: "sys7 HD Cleanup Utility" (was "Macintosh HD Cleanup Utility")

The System 7 *interaction design* (title bar chrome, 1-bit dithering, alert dialog layout) is a UI paradigm, not Apple-owned artwork, and isn't affected by this — only actual Apple assets/copy were replaced.

## Safety model

Deletion always goes through an allowlist-first path guard (`crates/sweep-core/src/safety.rs`): a path is only ever eligible for deletion if it canonicalizes under a registered scan target's root, checked *after* symlink resolution. Files are moved to the Trash by default; permanent deletion is a separate, explicitly gated action. See `crates/sweep-core/src/catalog.rs` for the full list of scan targets and which are safe to bulk-delete, require review, or are refused outright (e.g. Docker's disk image, iCloud Drive, Photos Library).
