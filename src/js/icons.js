// Minimal hard-edged (crispEdges, no antialiasing) placeholder glyphs for the
// icon grid. Real 1-bit System 7 icon art is an M7 asset task; these exist so
// the grid layout and size-label pairing can be built and tested now.
const FOLDER_ICON_SVG =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" shape-rendering="crispEdges">' +
      '<rect x="1" y="4" width="14" height="10" fill="#fff" stroke="#000" stroke-width="1"/>' +
      '<rect x="1" y="2" width="6" height="3" fill="#fff" stroke="#000" stroke-width="1"/>' +
      '</svg>'
  );

const DOCUMENT_ICON_SVG =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" shape-rendering="crispEdges">' +
      '<rect x="3" y="1" width="9" height="14" fill="#fff" stroke="#000" stroke-width="1"/>' +
      '<rect x="9" y="1" width="3" height="3" fill="#d8d8d8" stroke="#000" stroke-width="1"/>' +
      '</svg>'
  );

function iconForPath(path) {
  return path.endsWith("/") ? FOLDER_ICON_SVG : DOCUMENT_ICON_SVG;
}

// Caution glyph for destructive-action alerts: a hard-edged triangle with "!".
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
