# Third-party notices

Lynter is licensed under the MIT license in [LICENSE](LICENSE). It includes or is distributed with the third-party material listed here, each under its own license.

## Obsidian Linter documentation

Several Lynter rules are compatible with rules of the same name in [Obsidian Linter](https://github.com/platers/obsidian-linter), so that a vault can switch without changing its notes. Those rules were written from Obsidian Linter's published documentation. Their rule ids and option names follow the documented names, and some of the test examples in `crates/lynter-core/tests/fixtures/compat/` are copied from that documentation. Obsidian Linter and its documentation are used under the MIT license, with this copyright notice:

```text
Copyright (c) 2021-2022 Victor Tao
```

Lynter is not affiliated with or endorsed by Obsidian Linter or by Obsidian.

## Rust crates

The `lynter` binary and the WebAssembly build (used by the Obsidian plugin and by lynter.dev) include the crates listed in [packaging/licenses/rust-crates.txt](packaging/licenses/rust-crates.txt), with the license text of each. That file is generated from `Cargo.lock` by `packaging/licenses/generate.py`, checked in CI, and shipped inside every release archive.

## Fonts

lynter.dev serves Geist Sans and Geist Mono, copyright (c) 2023 Vercel, in collaboration with basement.studio, under the SIL Open Font License, Version 1.1. The license is in [packages/site/public/fonts/geist-license.txt](packages/site/public/fonts/geist-license.txt) and is served beside the fonts at <https://lynter.dev/fonts/geist-license.txt>.

The lappe.ai wordmark in the site footer is set in six glyphs of Jost, copyright 2020 The Jost Project Authors, under the SIL Open Font License, Version 1.1. The license is in [packages/site/public/fonts/jost-license.txt](packages/site/public/fonts/jost-license.txt) and is served beside the font at <https://lynter.dev/fonts/jost-license.txt>.

## JavaScript libraries

The Obsidian plugin bundle (`main.js`) includes:

| Library | License | Copyright notice |
| --- | --- | --- |
| [fflate](https://github.com/101arrowz/fflate) | MIT | Copyright (c) 2026 Arjun Barrett |

The lynter.dev pages include:

| Library | License | Copyright notice |
| --- | --- | --- |
| [React](https://github.com/facebook/react), React DOM and Scheduler | MIT | Copyright (c) Meta Platforms, Inc. and affiliates. |
| [Next.js](https://github.com/vercel/next.js) | MIT | Copyright (c) 2025 Vercel, Inc. |
| [Marked](https://github.com/markedjs/marked) | MIT | Copyright (c) 2018+, MarkedJS (https://github.com/markedjs/). Copyright (c) 2011-2018, Christopher Jeffrey (https://github.com/chjj/). |

Each is used under the MIT license with the copyright notice shown in its row.

## MIT license text

Every item above that is marked MIT is used under these terms, together with its own copyright notice as given in its section:

```text
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
