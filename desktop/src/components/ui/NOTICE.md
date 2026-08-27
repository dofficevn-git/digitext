# Vendored: shadcn/ui

The files in this directory were pulled in with the shadcn/ui CLI
(`npx shadcn add …`, style `new-york`) and then retuned locally against the
DigiText design tokens in `src/styles/theme.css` — 32px controls, 14px control
text, rectangular badges, a 2px focus ring, and measured-contrast colours in
place of upstream's `text-foreground/60`.

They are **not** DigiText-authored source, so they are exempt from the project
license-header gate (`dev/check-license-header.sh`) and from the
`react-refresh/only-export-components` lint rule (`eslint.config.js`). Upstream's
notice is retained below, as MIT requires.

Upstream: https://ui.shadcn.com — https://github.com/shadcn-ui/ui

```
MIT License

Copyright (c) 2023 shadcn

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
