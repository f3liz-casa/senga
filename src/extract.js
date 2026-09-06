// Runs inside the page. Walks the rendered tree and hands back one JSON string.
// Everything here is plain measurement; the judgement happens on the Rust side.
(() => {
  const SKIP = new Set(["SCRIPT", "STYLE", "NOSCRIPT", "TEMPLATE", "HEAD", "META", "LINK", "TITLE"]);
  const LIMIT = 4000;
  const boxes = [];

  const px = (v) => Math.round(parseFloat(v) || 0);
  const sides = (cs, name) => [
    px(cs[name + "Top"]), px(cs[name + "Right"]), px(cs[name + "Bottom"]), px(cs[name + "Left"]),
  ];
  const alpha = (rgb) => {
    const m = /rgba?\(([^)]+)\)/.exec(rgb);
    if (!m) return rgb === "transparent" ? 0 : 1;
    const parts = m[1].split(/[\s,\/]+/).filter(Boolean);
    return parts.length > 3 ? parseFloat(parts[3]) : 1;
  };
  const ownText = (el) => {
    let s = "";
    for (const n of el.childNodes) if (n.nodeType === 3) s += n.nodeValue;
    s = s.replace(/\s+/g, " ").trim();
    return s.length > 80 ? s.slice(0, 77) + "..." : s;
  };

  const walk = (el, parent, depth) => {
    if (boxes.length >= LIMIT || SKIP.has(el.tagName)) return;
    const cs = getComputedStyle(el);
    if (cs.display === "none" || cs.visibility === "hidden") return;
    const r = el.getBoundingClientRect();
    const x = r.left + scrollX, y = r.top + scrollY;
    const bg = cs.backgroundColor;
    const ownBg = alpha(bg) > 0;
    const hasBorder = ["Top", "Right", "Bottom", "Left"]
      .some((s) => px(cs["border" + s + "Width"]) > 0 && cs["border" + s + "Style"] !== "none");
    const overflow = cs.overflowX === "visible" && cs.overflowY === "visible" ? "visible" : cs.overflowX;
    const clipped = overflow !== "visible" &&
      (el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1);

    const i = boxes.length;
    boxes.push({
      parent, depth,
      tag: el.tagName.toLowerCase(),
      id: el.id || "",
      // Build-time hashes (svelte-1abc, css-x9y) say nothing about the design.
      cls: (typeof el.className === "string" ? el.className : "").trim().split(/\s+/)
        .filter((c) => c && !/^(svelte|css|sc|jsx|emotion)-[a-z0-9]+$/i.test(c) && !/^_[a-z0-9_]{5,}$/.test(c)).slice(0, 3),
      x: Math.round(x), y: Math.round(y), w: Math.round(r.width), h: Math.round(r.height),
      text: ownText(el),
      display: cs.display,
      position: cs.position,
      fontSize: px(cs.fontSize),
      fontWeight: parseInt(cs.fontWeight) || 400,
      color: cs.color,
      bg: ownBg ? bg : "",
      border: hasBorder,
      overflow, clipped,
      pad: sides(cs, "padding"),
      margin: sides(cs, "margin"),
      gap: cs.display.includes("flex") || cs.display.includes("grid") ? px(cs.gap || cs.rowGap) : 0,
      alt: el.tagName === "IMG" ? (el.getAttribute("alt") ?? null) : undefined,
    });
    for (const c of el.children) walk(c, i, depth + 1);
  };

  const rootBg = getComputedStyle(document.documentElement).backgroundColor;
  const canvasBg = alpha(rootBg) > 0 ? rootBg : (alpha(getComputedStyle(document.body).backgroundColor) > 0 ? getComputedStyle(document.body).backgroundColor : "rgb(255, 255, 255)");
  walk(document.body, -1, 0);

  return JSON.stringify({
    title: document.title,
    width: innerWidth, height: innerHeight,
    docWidth: document.documentElement.scrollWidth,
    docHeight: document.documentElement.scrollHeight,
    lang: document.documentElement.lang || "",
    canvasBg,
    boxes,
  });
})()
