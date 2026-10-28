(async () => {
  const rootPath = document.querySelector('meta[name="rustdoc-vars"]')?.dataset.rootPath;
  if (!rootPath) return;
  const root = new URL(rootPath, document.baseURI);
  const page = location.pathname.slice(root.pathname.length);
  if (!/^(library_checker|aizu_online_judge)\/.*\bfn\.[^/]+\.html$/.test(page)) return;
  const heading = document.querySelector("#main-content .main-heading");
  if (!heading) return;

  const status = document.createElement("span");
  status.setAttribute("role", "status");
  const read = async url => {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response;
  };
  try {
    const manifest = await (await read(new URL("submissions/index.json", root))).json();
    const source = manifest[page];
    if (!source) return;
    const toolbar = document.createElement("span");
    toolbar.style.cssText = "display:inline-flex;align-items:baseline;gap:.6em;margin-inline-start:1em";
    const button = document.createElement("button");
    button.type = "button";
    button.style.cssText = "font:inherit;color:var(--link-color);background:none;border:0;padding:0;cursor:pointer";
    button.title = "Copy submission code";
    button.setAttribute("aria-live", "polite");
    button.textContent = "Copy";
    button.disabled = true;
    const link = document.createElement("a");
    link.href = new URL(source, root).href;
    link.title = "View submission code";
    link.textContent = "Submission";
    status.textContent = "Loading…";
    toolbar.append(link, button, status);
    heading.querySelector(".sub-heading").append(toolbar);

    // Load before the click so clipboard access keeps the user's activation.
    const code = await (await read(link.href)).text();
    if (!code.trim()) throw new Error("Empty submission source");
    button.disabled = false;
    status.textContent = "";
    status.hidden = true;
    let copiedTimer;
    button.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(code);
        clearTimeout(copiedTimer);
        button.textContent = "Copied!";
        status.hidden = true;
        copiedTimer = setTimeout(() => {
          button.textContent = "Copy";
        }, 2000);
      } catch (error) {
        clearTimeout(copiedTimer);
        button.textContent = "Copy";
        status.textContent = "Could not copy. Open “Submission” and copy the source.";
        status.hidden = false;
        console.error("Submission clipboard:", error);
      }
    });
  } catch (error) {
    status.textContent = "Could not load submission code. Reload the page to retry.";
    if (!status.isConnected) heading.after(status);
    console.error("Submission source:", error);
  }
})();
