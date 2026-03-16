// SAF Global Navigation Bar — injected into mdBook
(function () {
  var nav = document.createElement('nav');
  nav.className = 'global-nav';

  // mdBook sets `path_to_root` as a global JS variable (e.g. "" for root, "../" for subpages).
  // This gives us the path back to /docs/ root. Go one more level up for site root.
  var docsRoot = (typeof path_to_root !== 'undefined' ? path_to_root : './') || './';
  var siteRoot = docsRoot + '../';

  nav.innerHTML =
    '<a href="' + siteRoot + '" class="global-nav-brand">SAF</a>' +
    '<div class="global-nav-links">' +
    '<a href="' + siteRoot + '">Home</a>' +
    '<a href="' + siteRoot + '#architecture">Architecture</a>' +
    '<a href="' + siteRoot + 'tutorials/">Tutorials</a>' +
    '<a href="' + siteRoot + 'playground/">Playground</a>' +
    '<a href="' + docsRoot + '" class="global-nav-active">Docs</a>' +
    '<a href="https://github.com" target="_blank" rel="noopener noreferrer">GitHub</a>' +
    '</div>';

  document.body.insertBefore(nav, document.body.firstChild);
})();
