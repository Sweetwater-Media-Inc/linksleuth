import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const mainSource = await readFile(new URL("../src/main.js", import.meta.url), "utf8");
const htmlSource = await readFile(new URL("../index.html", import.meta.url), "utf8");

test("frontend source avoids persistent storage, unsafe HTML, and input logging", () => {
  for (const forbidden of [
    ".innerHTML",
    "localStorage",
    "sessionStorage",
    "indexedDB",
    "document.cookie",
    "history.pushState",
    "history.replaceState",
    "console.log",
    "console.error",
  ]) {
    assert.equal(mainSource.includes(forbidden), false, `found ${forbidden}`);
  }
});

test("page has no remote asset or navigation declarations", () => {
  assert.equal(/(?:src|href)=["']https?:\/\//iu.test(htmlSource), false);
  assert.equal(/<(?:iframe|img|video|audio|source)\b/iu.test(htmlSource), false);
  assert.match(htmlSource, /Content-Security-Policy/u);
  assert.match(htmlSource, /connect-src 'self'/u);
});

test("destination evidence rendering cannot create navigation or embedded resources", () => {
  for (const forbidden of [
    'createElement("a"',
    "createElement('a'",
    ".href =",
    ".src =",
    'setAttribute("href"',
    'setAttribute("src"',
    "window.open",
    "location.assign",
    "location.replace",
  ]) {
    assert.equal(mainSource.includes(forbidden), false, `found ${forbidden}`);
  }
  assert.match(mainSource, /destination-reference/u);
  assert.match(mainSource, /\.textContent = candidate\.detail\.submitted_reference/u);
});
