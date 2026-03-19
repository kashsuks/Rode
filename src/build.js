const fs = require("fs");
const path = require("path");
const matter = require("gray-matter");
const { marked } = require("marked");

const SITE_DIR = path.join(__dirname, "_site");
const DOCS_DIR = path.join(__dirname, "docs");
const PUBLIC_DIR = path.join(__dirname, "public");
const PAGES_DIR = path.join(__dirname, "pages");

const siteConfig = {
  title: "Pinel",
  description: "Blazingly fast code editor in Rust",
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function copyDirSync(src, dest) {
  ensureDir(dest);
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirSync(srcPath, destPath);
    } else {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

function readTemplate(name) {
  return fs.readFileSync(path.join(__dirname, "templates", name), "utf-8");
}

function escapeHtml(str) {
  return str.replace(/[<>&"]/g, (c) => ({ "<": "&lt;", ">": "&gt;", "&": "&amp;", '"': "&quot;" }[c]));
}

function slugify(str) {
  return str.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
}

// ---------------------------------------------------------------------------
// Load docs from markdown
// ---------------------------------------------------------------------------

function loadDocs() {
  if (!fs.existsSync(DOCS_DIR)) return [];
  const files = fs.readdirSync(DOCS_DIR).filter((f) => f.endsWith(".md"));
  return files
    .map((file) => {
      const raw = fs.readFileSync(path.join(DOCS_DIR, file), "utf-8");
      const { data, content } = matter(raw);
      const slug = slugify(path.basename(file, ".md"));
      const html = marked.parse(content);
      // Strip HTML tags for search content
      const plainText = content.replace(/```[\s\S]*?```/g, (m) => m.replace(/```\w*\n?/g, "")).replace(/[#*`>\-\[\]()]/g, "").trim();
      return {
        slug,
        title: data.title || slug,
        description: data.description || "",
        category: data.category || "General",
        order: data.order || 999,
        html,
        plainText,
        url: `/docs/${slug}/`,
      };
    })
    .sort((a, b) => a.order - b.order);
}

// ---------------------------------------------------------------------------
// Build sidebar HTML
// ---------------------------------------------------------------------------

function buildSidebar(docs, activeSlug) {
  const categories = {};
  for (const doc of docs) {
    if (!categories[doc.category]) categories[doc.category] = [];
    categories[doc.category].push(doc);
  }

  let html = "";
  for (const [cat, items] of Object.entries(categories)) {
    html += `\n    <div class="sidebar-section">\n      <h4>${escapeHtml(cat)}</h4>\n      <ul>\n`;
    for (const item of items) {
      const active = item.slug === activeSlug ? ' class="active"' : "";
      html += `        <li><a href="${item.url}"${active}>${escapeHtml(item.title)}</a></li>\n`;
    }
    html += `      </ul>\n    </div>\n`;
  }
  return html;
}

// ---------------------------------------------------------------------------
// Build search data JS
// ---------------------------------------------------------------------------

function buildSearchData(docs) {
  return docs
    .map(
      (d) =>
        `  searchData.push(${JSON.stringify({
          title: d.title,
          url: d.url,
          content: d.plainText,
          category: d.category,
        })});`
    )
    .join("\n");
}

// ---------------------------------------------------------------------------
// Single docs file helpers (headings + sidebar)
// ---------------------------------------------------------------------------

function loadSingleDoc() {
  const file = path.join(DOCS_DIR, "docs.md");
  if (!fs.existsSync(file)) return null;
  const raw = fs.readFileSync(file, "utf-8");
  const { data, content } = matter(raw);

  const headings = [];
  const renderer = new marked.Renderer();
  renderer.heading = ({ text, depth }) => {
    const plain = typeof text === "string" ? text : String(text ?? "");
    const id = slugify(plain);
    headings.push({ level: depth, text: plain, id });
    return `<h${depth} id="${id}">${escapeHtml(plain)}</h${depth}>\n`;
  };

  const html = marked.parse(content, { renderer });
  const plainText = content
    .replace(/```[\s\S]*?```/g, (m) => m.replace(/```\w*\n?/g, ""))
    .replace(/[#*`>\-\[\]()]/g, "")
    .trim();

  return {
    title: data.title || "Documentation",
    description: data.description || "",
    html,
    plainText,
    headings,
    url: "/docs/",
    category: "Docs",
  };
}

function buildSidebarFromHeadings(headings) {
  if (!headings || headings.length === 0) return "";
  let html = '\n    <nav class="sidebar-tree">\n      <ul>\n';
  for (const h of headings) {
    const level = Math.min(Math.max(h.level, 1), 4);
    html += `        <li><a class="sidebar-link sidebar-link-level-${level}" href="#${h.id}">${escapeHtml(h.text)}</a></li>\n`;
  }
  html += "      </ul>\n    </nav>\n";
  return html;
}

// ---------------------------------------------------------------------------
// Render templates
// ---------------------------------------------------------------------------

function renderBase(vars) {
  let tpl = readTemplate("base.html");
  for (const [k, v] of Object.entries(vars)) {
    tpl = tpl.replace(new RegExp(`{{\\s*${k}\\s*}}`, "g"), v);
  }
  return tpl;
}

// ---------------------------------------------------------------------------
// Build
// ---------------------------------------------------------------------------

function build() {
  console.log("Building site...");

  // Clean
  if (fs.existsSync(SITE_DIR)) {
    fs.rmSync(SITE_DIR, { recursive: true });
  }
  ensureDir(SITE_DIR);

  // Copy public assets
  if (fs.existsSync(PUBLIC_DIR)) {
    copyDirSync(PUBLIC_DIR, SITE_DIR);
  }

  const singleDoc = loadSingleDoc();
  const docsForSearch = singleDoc ? [singleDoc] : [];
  const searchDataJs = buildSearchData(docsForSearch);
  const firstDocUrl = singleDoc ? singleDoc.url : "#";

  // --- Home page ---
  const homeBody = fs.readFileSync(path.join(PAGES_DIR, "index.html"), "utf-8");
  const homeHtml = renderBase({
    pageTitle: siteConfig.title,
    metaDescription: siteConfig.description,
    navHomeActive: ' class="active"',
    navInstallActive: "",
    navAboutActive: "",
    navDocsActive: "",
    body: homeBody,
    searchData: searchDataJs,
    firstDocUrl,
    siteTitle: siteConfig.title,
  });
  fs.writeFileSync(path.join(SITE_DIR, "index.html"), homeHtml);

  // --- Docs page ---
  if (singleDoc) {
    const docTemplate = readTemplate("docs.html");
    const sidebar = buildSidebarFromHeadings(singleDoc.headings);
    let docBody = docTemplate
      .replace("{{ sidebar }}", sidebar)
      .replace("{{ docTitle }}", escapeHtml(singleDoc.title))
      .replace("{{ docMeta }}", singleDoc.description ? `<div class="docs-meta">${escapeHtml(singleDoc.description)}</div>` : "")
      .replace("{{ docContent }}", singleDoc.html);

    const html = renderBase({
      pageTitle: `${singleDoc.title} — ${siteConfig.title}`,
      metaDescription: singleDoc.description || siteConfig.description,
      navHomeActive: "",
      navInstallActive: "",
      navAboutActive: "",
      navDocsActive: ' class="active"',
      body: docBody,
      searchData: searchDataJs,
      firstDocUrl,
      siteTitle: siteConfig.title,
    });

    const outDir = path.join(SITE_DIR, "docs");
    ensureDir(outDir);
    fs.writeFileSync(path.join(outDir, "index.html"), html);
  }

  // --- Install page ---
  const installBody = fs.readFileSync(path.join(PAGES_DIR, "install.html"), "utf-8");
  const installHtml = renderBase({
    pageTitle: `Install — ${siteConfig.title}`,
    metaDescription: "Install Pinel on macOS, Linux, or Windows",
    navHomeActive: "",
    navInstallActive: ' class="active"',
    navAboutActive: "",
    navDocsActive: "",
    body: installBody,
    searchData: searchDataJs,
    firstDocUrl,
    siteTitle: siteConfig.title,
  });
  fs.writeFileSync(path.join(SITE_DIR, "install.html"), installHtml);

  // --- About page ---
  const aboutBody = fs.readFileSync(path.join(PAGES_DIR, "about.html"), "utf-8");
  const aboutHtml = renderBase({
    pageTitle: `About — ${siteConfig.title}`,
    metaDescription: "Learn more about Pinel",
    navHomeActive: "",
    navInstallActive: "",
    navAboutActive: ' class="active"',
    navDocsActive: "",
    body: aboutBody,
    searchData: searchDataJs,
    firstDocUrl,
    siteTitle: siteConfig.title,
  });
  fs.writeFileSync(path.join(SITE_DIR, "about.html"), aboutHtml);

  // Copy installer script so it can be fetched via curl during setup
  const installerSrc = path.join(PAGES_DIR, "install.sh");
  if (fs.existsSync(installerSrc)) {
    const installerDest = path.join(SITE_DIR, "install.sh");
    fs.copyFileSync(installerSrc, installerDest);
    fs.chmodSync(installerDest, 0o755);
  }

  console.log("Built site → _site/ (home, install, about, docs)");
}

// ---------------------------------------------------------------------------
// Watch mode (--watch)
// ---------------------------------------------------------------------------

if (process.argv.includes("--watch")) {
  build();
  console.log("Watching for changes...");
  const watchDirs = [DOCS_DIR, PAGES_DIR, PUBLIC_DIR, path.join(__dirname, "templates")];
  for (const dir of watchDirs) {
    if (fs.existsSync(dir)) {
      fs.watch(dir, { recursive: true }, () => {
        try {
          build();
        } catch (e) {
          console.error("Build error:", e.message);
        }
      });
    }
  }
} else {
  build();
}
