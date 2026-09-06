# Layout dump

What `senga --json` prints, and what `src/render.rs` reads. It is produced by `src/extract.js`, which runs inside the page after layout has settled. All lengths are CSS px relative to the document (scroll offsets already added), rounded to integers.

```json
{
  "title": "板のサンプル",
  "width": 800, "height": 600,
  "docWidth": 980, "docHeight": 682,
  "lang": "ja",
  "canvasBg": "rgb(255, 255, 255)",
  "boxes": [ ... ]
}
```

| Field | Meaning |
|---|---|
| `width`, `height` | viewport |
| `docWidth`, `docHeight` | `documentElement.scrollWidth/Height`; wider than the viewport means a horizontal scrollbar |
| `lang` | `<html lang>` |
| `canvasBg` | what shows where nothing is painted: `<html>`'s background, else `<body>`'s, else white |
| `boxes` | every rendered element from `<body>` down, in document order (parents before children) |

## Box

```json
{
  "parent": 4, "depth": 2,
  "tag": "p", "id": "", "cls": ["lead"],
  "x": 80, "y": 160, "w": 640, "h": 24,
  "text": "ここは、書いた人を隠さない板です。",
  "display": "block", "position": "static",
  "fontSize": 16, "fontWeight": 400,
  "color": "rgb(153, 153, 153)",
  "bg": "",
  "border": false,
  "overflow": "visible", "clipped": false,
  "pad": [0, 0, 0, 0], "margin": [0, 0, 24, 0], "gap": 0,
  "alt": null
}
```

| Field | Meaning |
|---|---|
| `parent` | index into `boxes` of the parent element, `-1` for `<body>` |
| `depth` | nesting depth from `<body>` (0) |
| `tag`, `id`, `cls` | lowercase tag, `id`, up to three class names; build hashes like `svelte-1abc` or `css-x9y` are dropped |
| `x`, `y`, `w`, `h` | border box from `getBoundingClientRect`, plus scroll offset |
| `text` | the element's own text nodes only, whitespace collapsed, cut at 80 characters; children's text is on the children |
| `display`, `position` | computed values |
| `fontSize`, `fontWeight` | computed, px and numeric weight |
| `color` | computed text colour, as the engine serialises it: `rgb()`, `rgba()`, or `oklch()`/`oklab()` when the stylesheet used those |
| `bg` | own background colour, or `""` when transparent |
| `border` | any side has a visible border |
| `overflow` | `"visible"` or the computed `overflow-x` |
| `clipped` | `overflow` is not visible and scroll size exceeds client size by more than 1 px |
| `pad`, `margin` | `[top, right, bottom, left]` |
| `gap` | `gap` for flex and grid containers, else 0 |
| `alt` | only on `<img>`: the attribute, or `null` when absent |

The dump carries only each element's own background. The renderer works out what is really behind each one (`Page::settle`): an opaque background, a translucent one blended over what is behind its parent, or the parent's. Contrast is computed against that.

## What is left out

- `display: none` and `visibility: hidden` subtrees.
- `<script>`, `<style>`, `<template>`, `<noscript>` and the `<head>`.
- Pseudo-elements and text runs inside a text node; the box is the element's, not each line's.
- Shadow DOM contents (open shadow roots are not descended into yet).
- Anything past the 4000th element.

## Feeding it from another engine

The renderer only needs this shape. A Playwright script that runs `src/extract.js` in Firefox or Chromium and writes the string it returns produces a dump senga can render:

```
cat dump.json | senga-render     # not built yet; `senga::render_with` in src/lib.rs is the entry point
```

Comparing the same page's dump from Servo and from Firefox is a cheap way to find layout differences, some of which are Servo bugs.
