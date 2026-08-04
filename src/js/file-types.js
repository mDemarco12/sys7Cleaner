// Extension -> icon-type lookup, the single source of truth for which of the
// five file-type glyphs (see icons.js) a given path gets. Plain global
// function, matching how the rest of src/js/* works — no bundler, no ES
// modules, just ordered <script> tags sharing globals (see index.html).
const FILE_TYPE_EXTENSIONS = {
  photo: new Set([
    "jpg", "jpeg", "png", "gif", "heic", "heif", "webp", "tiff", "tif", "bmp",
    "avif", "dng", "raw", "cr2", "nef", "arw", "svg", "ico", "icns",
  ]),
  script: new Set([
    "sh", "bash", "zsh", "fish", "py", "rb", "pl", "lua", "js", "mjs", "cjs",
    "ts", "tsx", "jsx", "ps1", "bat", "cmd", "vbs", "r", "sql",
  ]),
  app: new Set([
    "exe", "app", "dmg", "pkg", "msi", "deb", "rpm", "appimage", "apk", "jar", "car",
  ]),
  // svg is deliberately in `photo` above, not here — users think of it as an
  // image, not markup. Don't "fix" this back to xml.
  xml: new Set([
    "xml", "plist", "json", "yaml", "yml", "toml", "html", "htm", "css", "ini", "cfg", "conf", "md",
  ]),
};

const EXTENSION_TO_TYPE = new Map();
for (const [type, extensions] of Object.entries(FILE_TYPE_EXTENSIONS)) {
  for (const ext of extensions) EXTENSION_TO_TYPE.set(ext, type);
}

// path -> "doc" | "photo" | "script" | "app" | "xml". A name with no dot, or
// a leading-dot dotfile like ".gitignore" (nothing after its only dot's
// position at index 0), has no extension and falls back to "doc" — same as
// any extension that isn't in the map above.
function fileTypeForPath(path) {
  const name = path.replace(/\/+$/, "").split("/").pop() || "";
  const dot = name.lastIndexOf(".");
  if (dot <= 0) return "doc";
  const ext = name.slice(dot + 1).toLowerCase();
  return EXTENSION_TO_TYPE.get(ext) || "doc";
}
