# senga (線画)

HTML+CSS in, text out. A page renderer for readers that measure better than they see.

Servo lays the page out. `senga` asks it where every box landed, then writes that down as:

- a **wireframe** in box-drawing characters, so the shape of the page is visible in a terminal or a chat,
- an **element list** with positions, sizes, font sizes, colours and contrast ratios,
- **findings**: horizontal overflow, children spilling out of parents, overlapping siblings, clipped content, low contrast, tiny text, missing `alt`, small tap targets,
- a **palette** of the font sizes, colours and spacing values actually in use, with a note on what falls off the 4 px grid.

It exists because an AI (or a tired human) reading a screenshot cannot tell 4 px from 8 px, nor 3.9:1 from 4.5:1. Numbers can.

## Usage

```
senga page.html
senga https://example.org/ --width 390 --height 844
senga page.html --png shot.png       # also save what Servo painted
senga page.html --json               # the raw layout dump, for other tools
senga page.html --cols 80 --wait 500 # narrower wireframe; wait 500 ms after load
```

The wireframe maps `width / cols` CSS px to one character, and twice that to one row. Solid frames are containers and widgets, named on their top edge. Dotted frames are text (headings, paragraphs, links) with their words inside. `╌` marks the fold of the first screen.

## Building

Servo is pulled in as a path dependency from a sibling checkout:

```
git clone --depth 1 https://github.com/servo/servo ../servo
brew install cmake pkg-config        # what Servo's own bootstrap installs on macOS
cargo build --release                # rustup fetches the toolchain Servo pins
```

The first build compiles SpiderMonkey and the rest of Servo (around ten minutes on an M4). Afterwards only this crate rebuilds. The binary is a single file; no browser needs to be installed.

On macOS the software GL context prints one `UNSUPPORTED (log once)` line on stderr. It is harmless.

## What it is not

It renders with Servo, not with Firefox or Chrome. Servo's CSS support is real but not complete, and its fonts differ, so line breaks and text heights can drift by a few pixels from what users see. Treat the numbers as the truth about the layout rules you wrote, and confirm the final look in the browsers people use. Differences between the two are sometimes Servo bugs worth reporting.

## Layout dump

`--json` prints what `src/extract.js` collected inside the page: for each element its parent, depth, tag, id, classes (build hashes stripped), box, own text, display, position, font size and weight, colour, own and effective background, border, overflow, padding, margin and gap. Anything that can produce this shape can feed `senga::render`.
