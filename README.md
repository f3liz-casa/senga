# senga (線画)

HTML+CSS in, text out. A page renderer for readers that measure better than they see.

Servo lays the page out. `senga` asks it where every box landed, then writes that down as:

- a **wireframe** in box-drawing characters, so the shape of the page is visible in a terminal or a chat,
- an **element list** with positions, sizes, font sizes, colours and contrast ratios,
- **findings**: horizontal overflow, children spilling out of parents, overlapping siblings, clipped content, low contrast, tiny text, missing `alt`, small tap targets,
- a **palette** of the font sizes, colours and spacing values actually in use, with a note on what falls off the 4 px grid,
- the **console**, because an empty wireframe is usually explained by one error line.

It exists because an AI (or a tired human) reading a screenshot cannot tell 4 px from 8 px, nor 3.9:1 from 4.5:1. Numbers can.

```
$ senga examples/board.html --width 800 --height 600 --cols 60
# 板のサンプル
viewport 800×600, document 980×682, 19 elements, lang=ja

## Wireframe  (1 char = 13x27 px, `╌` = fold of the first screen)
┌header─────────────────────────────────────────────────────┐
│ ·a····                                                a   │
└─······────────────────────────────────────────────────────┘

     ┌ h1 "ようこそ、板へ"·····························┐
     │·················································│
     │·p.lead··········································│
     │·················································│
     │┌article.card─·········┐─┌article.card··········┐│
     ││·h2 "旅の記録"       ·│ │·h2 "哲学デコ"       ·││
     ││┌img─────────────────┐│ │······················││
     │││                    ││ │·p                   ·││
     │││                    ││ │······················││
     │││                    ││ │┌butt┐                ││
     ││└────────────────────┘│ │└────┘                ││
     ││······················│ │                      ││
     ││·p                   ·│ │                      ││
     │┌div.wide──────────────────────────────────────────────
─────││·─────               ·                          │
     └└──────────────────────────────────────────────────────



┌footer─────────────────────────────────────────────────────┐
│                                                           │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

## Findings
- Horizontal scroll: document is 980 px wide in a 800 px viewport.
- `div.wide` reaches x=980, past the viewport edge at 800.
- `div.wide` spills out of `main` by 244 px horizontally.
- `article.card` clips its content (overflow: hidden).
- Low contrast 2.2:1 on `footer` (#aaaaaa on #fafafa) "© natadeco".
- Low contrast 2.8:1 on `p.lead` (#999999 on #ffffff) "ここは、書いた人を隠さない板です。加入は軽く、貨幣は作らない。".
- Low contrast 3.9:1 on `a.dim` (#777777 on #1a1a1a) "Login".
- Text under 12 px: `button` (11px).
- 1 image(s) without an alt attribute.
- Targets under 24 px: `a` 70×16, `a.dim` 34×14.
```

## Findings
- Horizontal scroll: document is 980 px wide in a 800 px viewport.
- `div.wide` reaches x=980, past the viewport edge at 800.
- `div.wide` spills out of `main` by 244 px horizontally.
- `article.card` clips its content (overflow: hidden).
- Low contrast 2.2:1 on `footer` (#aaaaaa on #fafafa) "© natadeco".
- Low contrast 2.8:1 on `p.lead` (#999999 on #ffffff) "ここは、書いた人を隠さない板です。加入は軽く、貨幣は作らない。".
- Low contrast 3.9:1 on `a.dim` (#777777 on #1a1a1a) "Login".
- Text under 12 px: `button` (11px).
- 1 image(s) without an alt attribute.
- Targets under 24 px: `a` 70×16, `a.dim` 34×14.
```

## Setup

senga is a Rust crate that embeds [Servo](https://servo.org) as a library. Servo is not on crates.io, so it is used from a sibling checkout as a path dependency, pinned to the commit in `SERVO_COMMIT`.

```
git clone https://github.com/f3liz-casa/senga && cd senga
scripts/fetch-servo.sh          # shallow clone of Servo into ../servo, at the pinned commit
brew install cmake pkg-config   # macOS; what Servo's own `mach bootstrap` installs
cargo build --release
```

Things worth knowing before the first build:

| | |
|---|---|
| Servo checkout | about 1.6 GB (shallow), at `../servo` by default (`SERVO_DIR=...` to move it) |
| Rust toolchain | the version in `rust-toolchain.toml`, fetched by rustup on first use (Servo pins it; senga follows) |
| First build | around ten minutes on an M4, compiling SpiderMonkey and all of Servo; `target/` ends up around 3.5 GB |
| Later builds | seconds, only this crate |
| Binary | one file, about 130 MB; no browser needs to be installed |
| Tested on | macOS (Apple Silicon). Linux should work with Servo's usual build deps, but has not been tried |

`.cargo/config.toml` carries the two environment variables Servo's own workspace sets (`RUSTC_BOOTSTRAP` for the crates that use nightly features on a stable compiler, and `MACOSX_DEPLOYMENT_TARGET`). They are required; without them the build fails deep inside Servo.

### Updating Servo

Servo's embedding API is still moving. To move senga to a newer Servo:

```
cd ../servo && git fetch --depth 1 origin main && git checkout --detach FETCH_HEAD
git rev-parse HEAD > ../senga/SERVO_COMMIT
cd ../senga && cargo build --release
```

Fix whatever no longer compiles in `src/main.rs` (it is the only file that touches Servo), run the pages under `examples/`, and commit `SERVO_COMMIT` together with the fix. The Servo pieces senga relies on: `SoftwareRenderingContext`, `ServoBuilder`, `WebViewBuilder`, `WebViewDelegate::{notify_load_status_changed, notify_new_frame_ready, show_console_message}`, `WebView::take_screenshot`, `WebView::evaluate_javascript`. Servo's own `components/servo/tests/` uses the same surface and is the best reference when something changed.

### Servo features

`Cargo.toml` enables `webcrypto` on top of Servo's defaults. It is off in Servo's default set, and without it any app that touches `crypto.subtle` (OAuth PKCE, most auth libraries) throws before it mounts. Other optional pieces (`webgpu`, `webxr`, `media-gstreamer`) are left out; they make the build heavier and layout does not need them.

## Usage

```
senga page.html
senga https://example.org/ --width 390 --height 844
senga page.html --png shot.png       # also save what Servo painted
senga page.html --json               # the raw layout dump, for other tools
senga page.html --cols 80            # narrower wireframe
senga app/index.html --wait 1500     # wait after `load` for a client-side app to settle
```

| Option | Default | Meaning |
|---|---|---|
| `--width`, `--height` | 1280, 800 | viewport in CSS px; the document may be taller |
| `--cols` | 100 | characters across the wireframe; one character is `width / cols` px, one row twice that |
| `--wait` | 0 | milliseconds to keep the event loop running after `load`, for hydration and fetched content |
| `--png` | | write Servo's own painting of the viewport |
| `--json` | | print the layout dump instead of the text report |

A local file path or any URL works. Loading goes through Servo's own network stack, so `https` and redirects behave as in a browser.

### Reading the output

**Wireframe.** Solid frames are containers and widgets (`header`, `nav`, `section`, `form`, `button`, `input`, `img`...), named on their top edge as `tag#id.class`. Dotted frames are text (headings, paragraphs, links, labels) with their words inside. Boxes too small for a frame get just a label. `╌` marks the fold of the first screen. Choose `--cols` so the smallest element you care about is at least six characters wide; for a 1280 px page that is `--cols 160`.

**Elements.** One line per drawn or text-bearing element, indented by depth: `x y w×h`, then display and position when not the default, own background, and for text: font size, weight, colour, effective background and the contrast ratio between them. `CLIPPED` marks an element that hides part of its content.

**Findings.** Each line is a measurement, not an opinion. Contrast thresholds follow WCAG AA (4.5:1, or 3:1 for text at 24 px or bold 19 px). Tap targets are flagged under 24 px. Off-grid spacing is reported in the palette, not as a finding, because it is often deliberate.

**Console.** Everything the page logged while loading. When the wireframe is nearly empty, read this first.

## Layout dump

`--json` prints what `src/extract.js` collected inside the page. The text renderer knows nothing about Servo; anything that can produce this shape can feed it. The schema is described in [docs/layout-dump.md](docs/layout-dump.md).

## What it is not

It renders with Servo, not with Firefox or Chrome. Servo's CSS support is real but not complete, its fonts differ, and some web APIs are missing (kaguya logs `Locks API not available`, for example). Line breaks and text heights can drift by a few pixels from what users see. Treat the numbers as the truth about the layout rules you wrote, and confirm the final look in the browsers people use. When the two disagree, it is sometimes a Servo bug worth reporting.

It measures; it does not judge. A page can pass every finding and still be ugly, and a 2.3:1 title can be the right call for a wordmark.

On macOS the software GL context prints one `UNSUPPORTED (log once)` line on stderr. It is harmless.

## Layout of this repository

```
src/main.rs      drives Servo: headless context, load, settle, run extract.js, print
src/extract.js   runs inside the page, produces the layout dump
src/render.rs    layout dump to text: wireframe, findings, list, palette
src/lib.rs       exposes render for other front ends
scripts/fetch-servo.sh   shallow clone of Servo at SERVO_COMMIT
SERVO_COMMIT     the Servo commit senga is built against
examples/        small pages that exercise the findings
docs/            layout dump schema
```
