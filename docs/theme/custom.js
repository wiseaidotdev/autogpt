(function () {
  document.addEventListener("DOMContentLoaded", function () {
    document.querySelectorAll("pre code").forEach(function (block) {
      const btn = document.createElement("button");
      btn.className = "copy-btn";
      btn.setAttribute("aria-label", "Copy to clipboard");
      btn.innerHTML = "⎘ Copy";
      btn.style.cssText = [
        "position:absolute",
        "top:0.5rem",
        "right:0.5rem",
        "padding:0.2rem 0.55rem",
        "font-size:0.72rem",
        "font-family:inherit",
        "border-radius:4px",
        "border:1px solid rgba(224,90,0,0.4)",
        "background:rgba(224,90,0,0.1)",
        "color:var(--brand-primary,#e05a00)",
        "cursor:pointer",
        "transition:background 0.2s",
        "z-index:5",
      ].join(";");

      const pre = block.parentElement;
      if (pre && pre.tagName === "PRE") {
        pre.style.position = "relative";
        pre.appendChild(btn);
      }

      btn.addEventListener("click", function () {
        navigator.clipboard.writeText(block.innerText).then(function () {
          btn.innerHTML = "✓ Copied!";
          btn.style.background = "rgba(16,185,129,0.15)";
          btn.style.borderColor = "#10b981";
          btn.style.color = "#10b981";
          setTimeout(function () {
            btn.innerHTML = "⎘ Copy";
            btn.style.background = "rgba(224,90,0,0.1)";
            btn.style.borderColor = "rgba(224,90,0,0.4)";
            btn.style.color = "var(--brand-primary,#e05a00)";
          }, 2000);
        });
      });
    });

    document.querySelectorAll('a[href^="#"]').forEach(function (anchor) {
      anchor.addEventListener("click", function (e) {
        const target = document.querySelector(anchor.getAttribute("href"));
        if (target) {
          e.preventDefault();
          target.scrollIntoView({ behavior: "smooth", block: "start" });
        }
      });
    });

    const currentUrl = window.location.href
      .split("#")[0]
      .replace(/\/$/, "")
      .replace(/\.html$/, "");
    document.querySelectorAll(".sidebar a").forEach(function (link) {
      const linkUrl = link.href
        .split("#")[0]
        .replace(/\/$/, "")
        .replace(/\.html$/, "");
      if (linkUrl === currentUrl) {
        document
          .querySelectorAll(".chapter li.chapter-item.active")
          .forEach((li) => li.classList.remove("active"));
        link.parentElement.classList.add("active");
      }
    });
  });
})();
