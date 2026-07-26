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
