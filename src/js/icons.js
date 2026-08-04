// Hard-edged (crispEdges, no antialiasing) 1-bit glyphs for the icon grid.
// Folder and file-type icons are inline SVG *markup strings*, not data
// URIs — an <img src="data:image/svg+xml..."> is an isolated document that
// can't inherit `color` from the host page, so fill="currentColor" would
// never see the cell's real color and could never invert on selection. app.js
// sets these directly via innerHTML on a wrapper element instead of img.src.
//
// The five file-type glyphs mirror icons/*.svg 1:1 (see build_icons.py for
// the pixel-grid source of truth for photo/script/app/doc; icons/xml.svg was
// authored directly in the same house style since no XML asset was supplied
// — swap in a real one there and re-transcribe here if that changes). Keep
// these in sync with the committed .svg files if either ever changes.

const DOC_ICON_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" shape-rendering="crispEdges" role="img" aria-label="doc"><g fill="currentColor"><rect x="6" y="3" width="20" height="1"/><rect x="6" y="4" width="1" height="1"/><rect x="25" y="4" width="1" height="1"/><rect x="6" y="5" width="1" height="1"/><rect x="20" y="5" width="6" height="1"/><rect x="6" y="6" width="1" height="1"/><rect x="20" y="6" width="6" height="1"/><rect x="6" y="7" width="1" height="1"/><rect x="20" y="7" width="6" height="1"/><rect x="6" y="8" width="1" height="1"/><rect x="20" y="8" width="6" height="1"/><rect x="6" y="9" width="1" height="1"/><rect x="20" y="9" width="6" height="1"/><rect x="6" y="10" width="1" height="1"/><rect x="25" y="10" width="1" height="1"/><rect x="6" y="11" width="1" height="1"/><rect x="25" y="11" width="1" height="1"/><rect x="6" y="12" width="1" height="1"/><rect x="25" y="12" width="1" height="1"/><rect x="6" y="13" width="1" height="1"/><rect x="25" y="13" width="1" height="1"/><rect x="6" y="14" width="1" height="1"/><rect x="25" y="14" width="1" height="1"/><rect x="6" y="15" width="1" height="1"/><rect x="25" y="15" width="1" height="1"/><rect x="6" y="16" width="1" height="1"/><rect x="25" y="16" width="1" height="1"/><rect x="6" y="17" width="1" height="1"/><rect x="25" y="17" width="1" height="1"/><rect x="6" y="18" width="1" height="1"/><rect x="25" y="18" width="1" height="1"/><rect x="6" y="19" width="1" height="1"/><rect x="25" y="19" width="1" height="1"/><rect x="6" y="20" width="1" height="1"/><rect x="25" y="20" width="1" height="1"/><rect x="6" y="21" width="1" height="1"/><rect x="25" y="21" width="1" height="1"/><rect x="6" y="22" width="1" height="1"/><rect x="25" y="22" width="1" height="1"/><rect x="6" y="23" width="1" height="1"/><rect x="25" y="23" width="1" height="1"/><rect x="6" y="24" width="1" height="1"/><rect x="25" y="24" width="1" height="1"/><rect x="6" y="25" width="1" height="1"/><rect x="25" y="25" width="1" height="1"/><rect x="6" y="26" width="1" height="1"/><rect x="25" y="26" width="1" height="1"/><rect x="6" y="27" width="1" height="1"/><rect x="25" y="27" width="1" height="1"/><rect x="6" y="28" width="20" height="1"/></g></svg>';

const PHOTO_ICON_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" shape-rendering="crispEdges" role="img" aria-label="photo"><g fill="currentColor"><rect x="6" y="3" width="20" height="1"/><rect x="6" y="4" width="1" height="1"/><rect x="25" y="4" width="1" height="1"/><rect x="6" y="5" width="1" height="1"/><rect x="20" y="5" width="6" height="1"/><rect x="6" y="6" width="1" height="1"/><rect x="20" y="6" width="6" height="1"/><rect x="6" y="7" width="1" height="1"/><rect x="20" y="7" width="6" height="1"/><rect x="6" y="8" width="1" height="1"/><rect x="20" y="8" width="6" height="1"/><rect x="6" y="9" width="1" height="1"/><rect x="20" y="9" width="6" height="1"/><rect x="6" y="10" width="1" height="1"/><rect x="25" y="10" width="1" height="1"/><rect x="6" y="11" width="1" height="1"/><rect x="10" y="11" width="2" height="1"/><rect x="25" y="11" width="1" height="1"/><rect x="6" y="12" width="1" height="1"/><rect x="9" y="12" width="4" height="1"/><rect x="25" y="12" width="1" height="1"/><rect x="6" y="13" width="1" height="1"/><rect x="9" y="13" width="4" height="1"/><rect x="25" y="13" width="1" height="1"/><rect x="6" y="14" width="1" height="1"/><rect x="10" y="14" width="2" height="1"/><rect x="25" y="14" width="1" height="1"/><rect x="6" y="15" width="1" height="1"/><rect x="25" y="15" width="1" height="1"/><rect x="6" y="16" width="1" height="1"/><rect x="25" y="16" width="1" height="1"/><rect x="6" y="17" width="1" height="1"/><rect x="16" y="17" width="1" height="1"/><rect x="25" y="17" width="1" height="1"/><rect x="6" y="18" width="1" height="1"/><rect x="15" y="18" width="3" height="1"/><rect x="25" y="18" width="1" height="1"/><rect x="6" y="19" width="1" height="1"/><rect x="14" y="19" width="5" height="1"/><rect x="25" y="19" width="1" height="1"/><rect x="6" y="20" width="1" height="1"/><rect x="11" y="20" width="1" height="1"/><rect x="13" y="20" width="7" height="1"/><rect x="25" y="20" width="1" height="1"/><rect x="6" y="21" width="1" height="1"/><rect x="10" y="21" width="11" height="1"/><rect x="25" y="21" width="1" height="1"/><rect x="6" y="22" width="1" height="1"/><rect x="9" y="22" width="13" height="1"/><rect x="25" y="22" width="1" height="1"/><rect x="6" y="23" width="1" height="1"/><rect x="8" y="23" width="15" height="1"/><rect x="25" y="23" width="1" height="1"/><rect x="6" y="24" width="1" height="1"/><rect x="8" y="24" width="16" height="1"/><rect x="25" y="24" width="1" height="1"/><rect x="6" y="25" width="1" height="1"/><rect x="25" y="25" width="1" height="1"/><rect x="6" y="26" width="1" height="1"/><rect x="25" y="26" width="1" height="1"/><rect x="6" y="27" width="1" height="1"/><rect x="25" y="27" width="1" height="1"/><rect x="6" y="28" width="20" height="1"/></g></svg>';

const SCRIPT_ICON_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" shape-rendering="crispEdges" role="img" aria-label="script"><g fill="currentColor"><rect x="6" y="3" width="20" height="1"/><rect x="6" y="4" width="1" height="1"/><rect x="25" y="4" width="1" height="1"/><rect x="6" y="5" width="1" height="1"/><rect x="20" y="5" width="6" height="1"/><rect x="6" y="6" width="1" height="1"/><rect x="20" y="6" width="6" height="1"/><rect x="6" y="7" width="1" height="1"/><rect x="20" y="7" width="6" height="1"/><rect x="6" y="8" width="1" height="1"/><rect x="20" y="8" width="6" height="1"/><rect x="6" y="9" width="1" height="1"/><rect x="20" y="9" width="6" height="1"/><rect x="6" y="10" width="1" height="1"/><rect x="25" y="10" width="1" height="1"/><rect x="6" y="11" width="1" height="1"/><rect x="9" y="11" width="2" height="1"/><rect x="25" y="11" width="1" height="1"/><rect x="6" y="12" width="1" height="1"/><rect x="10" y="12" width="2" height="1"/><rect x="25" y="12" width="1" height="1"/><rect x="6" y="13" width="1" height="1"/><rect x="11" y="13" width="2" height="1"/><rect x="25" y="13" width="1" height="1"/><rect x="6" y="14" width="1" height="1"/><rect x="10" y="14" width="2" height="1"/><rect x="14" y="14" width="6" height="1"/><rect x="25" y="14" width="1" height="1"/><rect x="6" y="15" width="1" height="1"/><rect x="9" y="15" width="2" height="1"/><rect x="14" y="15" width="6" height="1"/><rect x="25" y="15" width="1" height="1"/><rect x="6" y="16" width="1" height="1"/><rect x="25" y="16" width="1" height="1"/><rect x="6" y="17" width="1" height="1"/><rect x="25" y="17" width="1" height="1"/><rect x="6" y="18" width="1" height="1"/><rect x="9" y="18" width="12" height="1"/><rect x="25" y="18" width="1" height="1"/><rect x="6" y="19" width="1" height="1"/><rect x="9" y="19" width="12" height="1"/><rect x="25" y="19" width="1" height="1"/><rect x="6" y="20" width="1" height="1"/><rect x="25" y="20" width="1" height="1"/><rect x="6" y="21" width="1" height="1"/><rect x="12" y="21" width="12" height="1"/><rect x="25" y="21" width="1" height="1"/><rect x="6" y="22" width="1" height="1"/><rect x="12" y="22" width="12" height="1"/><rect x="25" y="22" width="1" height="1"/><rect x="6" y="23" width="1" height="1"/><rect x="25" y="23" width="1" height="1"/><rect x="6" y="24" width="1" height="1"/><rect x="9" y="24" width="9" height="1"/><rect x="25" y="24" width="1" height="1"/><rect x="6" y="25" width="1" height="1"/><rect x="9" y="25" width="9" height="1"/><rect x="25" y="25" width="1" height="1"/><rect x="6" y="26" width="1" height="1"/><rect x="25" y="26" width="1" height="1"/><rect x="6" y="27" width="1" height="1"/><rect x="25" y="27" width="1" height="1"/><rect x="6" y="28" width="20" height="1"/></g></svg>';

const APP_ICON_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" shape-rendering="crispEdges" role="img" aria-label="app"><g fill="currentColor"><rect x="6" y="3" width="20" height="1"/><rect x="6" y="4" width="1" height="1"/><rect x="25" y="4" width="1" height="1"/><rect x="6" y="5" width="1" height="1"/><rect x="20" y="5" width="6" height="1"/><rect x="6" y="6" width="1" height="1"/><rect x="20" y="6" width="6" height="1"/><rect x="6" y="7" width="1" height="1"/><rect x="20" y="7" width="6" height="1"/><rect x="6" y="8" width="1" height="1"/><rect x="20" y="8" width="6" height="1"/><rect x="6" y="9" width="1" height="1"/><rect x="15" y="9" width="2" height="1"/><rect x="20" y="9" width="6" height="1"/><rect x="6" y="10" width="1" height="1"/><rect x="14" y="10" width="4" height="1"/><rect x="25" y="10" width="1" height="1"/><rect x="6" y="11" width="1" height="1"/><rect x="13" y="11" width="6" height="1"/><rect x="25" y="11" width="1" height="1"/><rect x="6" y="12" width="1" height="1"/><rect x="12" y="12" width="8" height="1"/><rect x="25" y="12" width="1" height="1"/><rect x="6" y="13" width="1" height="1"/><rect x="11" y="13" width="10" height="1"/><rect x="25" y="13" width="1" height="1"/><rect x="6" y="14" width="1" height="1"/><rect x="10" y="14" width="12" height="1"/><rect x="25" y="14" width="1" height="1"/><rect x="6" y="15" width="1" height="1"/><rect x="9" y="15" width="14" height="1"/><rect x="25" y="15" width="1" height="1"/><rect x="6" y="16" width="1" height="1"/><rect x="9" y="16" width="14" height="1"/><rect x="25" y="16" width="1" height="1"/><rect x="6" y="17" width="1" height="1"/><rect x="10" y="17" width="12" height="1"/><rect x="25" y="17" width="1" height="1"/><rect x="6" y="18" width="1" height="1"/><rect x="11" y="18" width="10" height="1"/><rect x="25" y="18" width="1" height="1"/><rect x="6" y="19" width="1" height="1"/><rect x="12" y="19" width="8" height="1"/><rect x="25" y="19" width="1" height="1"/><rect x="6" y="20" width="1" height="1"/><rect x="13" y="20" width="6" height="1"/><rect x="25" y="20" width="1" height="1"/><rect x="6" y="21" width="1" height="1"/><rect x="14" y="21" width="4" height="1"/><rect x="25" y="21" width="1" height="1"/><rect x="6" y="22" width="1" height="1"/><rect x="15" y="22" width="2" height="1"/><rect x="25" y="22" width="1" height="1"/><rect x="6" y="23" width="1" height="1"/><rect x="25" y="23" width="1" height="1"/><rect x="6" y="24" width="1" height="1"/><rect x="25" y="24" width="1" height="1"/><rect x="6" y="25" width="1" height="1"/><rect x="25" y="25" width="1" height="1"/><rect x="6" y="26" width="1" height="1"/><rect x="25" y="26" width="1" height="1"/><rect x="6" y="27" width="1" height="1"/><rect x="25" y="27" width="1" height="1"/><rect x="6" y="28" width="20" height="1"/></g></svg>';

// XML icon has no supplied asset — authored directly in the shared house
// style (same frame, same 1px-rect chevron construction app.svg's diamond
// uses) as a "</>" glyph. Swap in icons/xml.svg's real markup here if a
// proper asset is ever produced.
const XML_ICON_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" shape-rendering="crispEdges" role="img" aria-label="xml"><g fill="currentColor"><rect x="6" y="3" width="20" height="1"/><rect x="6" y="4" width="1" height="1"/><rect x="25" y="4" width="1" height="1"/><rect x="6" y="5" width="1" height="1"/><rect x="20" y="5" width="6" height="1"/><rect x="6" y="6" width="1" height="1"/><rect x="20" y="6" width="6" height="1"/><rect x="6" y="7" width="1" height="1"/><rect x="20" y="7" width="6" height="1"/><rect x="6" y="8" width="1" height="1"/><rect x="20" y="8" width="6" height="1"/><rect x="6" y="9" width="1" height="1"/><rect x="20" y="9" width="6" height="1"/><rect x="6" y="10" width="1" height="1"/><rect x="25" y="10" width="1" height="1"/><rect x="6" y="11" width="1" height="1"/><rect x="25" y="11" width="1" height="1"/><rect x="6" y="12" width="1" height="1"/><rect x="25" y="12" width="1" height="1"/><rect x="6" y="13" width="1" height="1"/><rect x="25" y="13" width="1" height="1"/><rect x="6" y="14" width="1" height="1"/><rect x="25" y="14" width="1" height="1"/><rect x="6" y="15" width="1" height="1"/><rect x="12" y="15" width="1" height="1"/><rect x="18" y="15" width="1" height="1"/><rect x="21" y="15" width="1" height="1"/><rect x="25" y="15" width="1" height="1"/><rect x="6" y="16" width="1" height="1"/><rect x="11" y="16" width="1" height="1"/><rect x="17" y="16" width="1" height="1"/><rect x="22" y="16" width="1" height="1"/><rect x="25" y="16" width="1" height="1"/><rect x="6" y="17" width="1" height="1"/><rect x="10" y="17" width="1" height="1"/><rect x="17" y="17" width="1" height="1"/><rect x="23" y="17" width="1" height="1"/><rect x="25" y="17" width="1" height="1"/><rect x="6" y="18" width="1" height="1"/><rect x="9" y="18" width="1" height="1"/><rect x="16" y="18" width="1" height="1"/><rect x="24" y="18" width="1" height="1"/><rect x="25" y="18" width="1" height="1"/><rect x="6" y="19" width="1" height="1"/><rect x="9" y="19" width="1" height="1"/><rect x="15" y="19" width="1" height="1"/><rect x="24" y="19" width="1" height="1"/><rect x="25" y="19" width="1" height="1"/><rect x="6" y="20" width="1" height="1"/><rect x="10" y="20" width="1" height="1"/><rect x="15" y="20" width="1" height="1"/><rect x="23" y="20" width="1" height="1"/><rect x="25" y="20" width="1" height="1"/><rect x="6" y="21" width="1" height="1"/><rect x="11" y="21" width="1" height="1"/><rect x="14" y="21" width="1" height="1"/><rect x="22" y="21" width="1" height="1"/><rect x="25" y="21" width="1" height="1"/><rect x="6" y="22" width="1" height="1"/><rect x="12" y="22" width="1" height="1"/><rect x="21" y="22" width="1" height="1"/><rect x="25" y="22" width="1" height="1"/><rect x="6" y="23" width="1" height="1"/><rect x="25" y="23" width="1" height="1"/><rect x="6" y="24" width="1" height="1"/><rect x="25" y="24" width="1" height="1"/><rect x="6" y="25" width="1" height="1"/><rect x="25" y="25" width="1" height="1"/><rect x="6" y="26" width="1" height="1"/><rect x="25" y="26" width="1" height="1"/><rect x="6" y="27" width="1" height="1"/><rect x="25" y="27" width="1" height="1"/><rect x="6" y="28" width="20" height="1"/></g></svg>';

const FILE_TYPE_ICONS = {
  doc: DOC_ICON_SVG,
  photo: PHOTO_ICON_SVG,
  script: SCRIPT_ICON_SVG,
  app: APP_ICON_SVG,
  xml: XML_ICON_SVG,
};

function fileIconSvg(type) {
  return FILE_TYPE_ICONS[type] || DOC_ICON_SVG;
}

// Faithful currentColor/32px conversion of the original 16px fill="#fff"
// stroke="#000" folder glyph (coordinates doubled, colors swapped for
// inheritance) — same silhouette as before, now able to invert on selection
// like the file-type glyphs above, and sharing their 32x32 box so every
// glyph in the grid sits on one consistent pixel scale.
const FOLDER_ICON_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" shape-rendering="crispEdges" role="img" aria-label="folder">' +
    '<rect x="2" y="8" width="28" height="20" fill="none" stroke="currentColor" stroke-width="1"/>' +
    '<rect x="2" y="4" width="12" height="6" fill="none" stroke="currentColor" stroke-width="1"/>' +
    '</svg>';

// path -> inline SVG markup (folder glyph for a trailing "/", otherwise the
// glyph matching the file's extension via fileTypeForPath, see file-types.js).
function iconForPath(path) {
  return path.endsWith("/") ? FOLDER_ICON_SVG : fileIconSvg(fileTypeForPath(path));
}

// Caution glyph for destructive-action alerts: a hard-edged triangle with "!".
// Rendered through an <img> (#alert-icon), not the grid glyph path, so it
// stays a self-contained data URI rather than currentColor markup.
const CAUTION_ICON_SVG =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" shape-rendering="crispEdges">' +
      '<polygon points="16,2 30,28 2,28" fill="#fff" stroke="#000" stroke-width="2"/>' +
      '<rect x="14.5" y="11" width="3" height="9" fill="#000"/>' +
      '<rect x="14.5" y="22" width="3" height="3" fill="#000"/>' +
      '</svg>'
  );

// Original angular dark-red emblem for the boot splash — deliberately its
// own geometry (not a trace of any existing insignia): an overlapping
// triangle/wedge crest in the "harsh angular mark" family, sized to sit
// where an OS vendor logo would, but owned outright by this project.
// Rendered through an <img> (#splash-logo); a self-contained data URI too.
const SYS7_EMBLEM_SVG =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">' +
      '<polygon points="50,4 82,34 82,66 50,96 18,66 18,34" fill="#6e0f22"/>' +
      '<polygon points="50,4 50,50 18,34" fill="#8a1730"/>' +
      '<polygon points="50,4 50,50 82,34" fill="#57091a"/>' +
      '<polygon points="18,66 50,50 50,96" fill="#57091a"/>' +
      '<polygon points="82,66 50,50 50,96" fill="#8a1730"/>' +
      '<polygon points="50,50 36,50 50,26" fill="#fff"/>' +
      '<polygon points="50,50 64,50 50,74" fill="#fff"/>' +
      '</svg>'
  );
