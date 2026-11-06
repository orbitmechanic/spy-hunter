#!/usr/bin/env node
// Browser smoke test for the staged SPY HUNTER web build.
//
//   node tools/verify_web.mjs [url]
//
// Boots Chromium headlessly with SwiftShader, waits for the wasm module,
// clicks the start button with a trusted CDP event (which unlocks audio),
// then drives Arrow keys + Space and captures the title, gameplay, and
// post-input frames. Fails on console errors or a missing wasm boot.

import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { setTimeout as sleep } from 'node:timers/promises';

const TARGET_URL = process.argv[2] || 'http://127.0.0.1:8123/';
const OUT = new URL('../shots/web/', import.meta.url).pathname;
mkdirSync(OUT, { recursive: true });

const chrome = spawn('chromium', [
  '--headless=new', '--no-sandbox',
  '--use-angle=swiftshader', '--enable-unsafe-swiftshader',
  '--remote-debugging-port=9224',
  '--remote-allow-origins=*',
  '--user-data-dir=/tmp/opencode/chrome-spy',
  '--window-size=1280,1200',
  'about:blank',
], { stdio: 'ignore' });

async function findTarget() {
  for (let i = 0; i < 120; i++) {
    try {
      const response = await fetch('http://127.0.0.1:9224/json/list');
      const list = await response.json();
      const page = list.find((target) => target.type === 'page');
      if (page) return page;
    } catch (_) {}
    await sleep(250);
  }
  throw new Error('Chromium did not expose a page target');
}

console.log('chromium target found');
const target = await findTarget();
const ws = new WebSocket(target.webSocketDebuggerUrl);
let nextId = 1;
const pending = new Map();
const consoleErrors = [];
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.id && pending.has(message.id)) {
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error(JSON.stringify(message.error)));
    else resolve(message.result);
  }
  if (message.method === 'Runtime.exceptionThrown') {
    consoleErrors.push(message.params.exceptionDetails?.exception?.description || 'exception');
  }
  if (message.method === 'Runtime.consoleAPICalled' && message.params.type === 'error') {
    consoleErrors.push(message.params.args.map((a) => a.value ?? a.description).join(' '));
  }
};
await new Promise((resolve, reject) => {
  ws.onopen = resolve;
  ws.onerror = reject;
});
console.log('devtools websocket open');

function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate(expression) {
  const result = await send('Runtime.evaluate', {
    expression, returnByValue: true, awaitPromise: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.exception?.description || result.exceptionDetails.text);
  }
  return result.result.value;
}

async function evaluateSoft(expression, ms = 4000) {
  return Promise.race([
    evaluate(expression).catch(() => null),
    sleep(ms).then(() => null),
  ]);
}

async function shot(name) {
  const { data } = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(`${OUT}/${name}.png`, Buffer.from(data, 'base64'));
}

await send('Page.enable');
await send('Runtime.enable');
await send('Network.enable');
await send('Network.setCacheDisabled', { cacheDisabled: true });
await send('Page.navigate', { url: TARGET_URL });
console.log('navigated');
await sleep(6000);

// Boot the 27 MB wasm. Runtime.evaluate can block while SwiftShader compiles,
// so every poll has a hard timeout.
let ready = false;
for (let i = 0; i < 120; i++) {
  ready = await evaluateSoft('window.__spyReady === true');
  if (ready) break;
  await sleep(500);
}
if (!ready) {
  await shot('boot-failure');
  const state = await evaluateSoft(`({
    ready: window.__spyReady,
    loadingHidden: document.getElementById('loading').hidden,
    startHidden: document.getElementById('start-overlay').hidden,
    errorHidden: document.getElementById('boot-error').hidden,
    errorText: document.getElementById('boot-error').innerText,
    audio: window.__spyAudioContext ? window.__spyAudioContext.state : 'none'
  })`);
  console.log('boot state', state);
  console.log('console errors', consoleErrors);
  throw new Error('wasm module never set window.__spyReady');
}
await sleep(1200);
await shot('title');

// Trusted click on the start button: unlocks AudioContext and dispatches Enter.
await evaluate(`document.getElementById('start-button').scrollIntoView({block: 'center'})`);
await sleep(400);
const rect = await evaluate(`(() => {
  const el = document.getElementById('start-button');
  const r = el.getBoundingClientRect();
  return { x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width, h: r.height };
})()`);
console.log('start button rect', rect);
for (const type of ['mousePressed', 'mouseReleased']) {
  await send('Input.dispatchMouseEvent', {
    type, x: rect.x, y: rect.y, button: 'left', clickCount: 1,
  });
}
let started = false;
for (let i = 0; i < 40; i++) {
  await sleep(250);
  started = await evaluateSoft('window.__spyStarted === true');
  if (started) break;
}
if (!started) throw new Error('start button did not register');
await sleep(2500);
await shot('gameplay');

// Drive the Interceptor: steer right, fire, then steer left.
async function key(code, keyName, vk, ms = 900) {
  await send('Input.dispatchKeyEvent', {
    type: 'keyDown', code, key: keyName, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk,
  });
  await sleep(ms);
  await send('Input.dispatchKeyEvent', {
    type: 'keyUp', code, key: keyName, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk,
  });
}
await key('ArrowRight', 'ArrowRight', 39, 900);
await send('Input.dispatchKeyEvent', {
  type: 'keyDown', code: 'Space', key: ' ', windowsVirtualKeyCode: 32, nativeVirtualKeyCode: 32,
});
await sleep(1400);
await send('Input.dispatchKeyEvent', {
  type: 'keyUp', code: 'Space', key: ' ', windowsVirtualKeyCode: 32, nativeVirtualKeyCode: 32,
});
await key('ArrowLeft', 'ArrowLeft', 37, 700);
await sleep(800);
await shot('after-input');

// Validate the JS -> wasm touch bridge without needing a touch device:
// dispatch pointer events on the on-screen steer/fire buttons.
await evaluate(`(() => {
  const el = document.querySelector('[data-key="ArrowRight"]');
  el.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, pointerId: 1 }));
})()`);
await sleep(900);
await evaluate(`(() => {
  const el = document.querySelector('[data-key="ArrowRight"]');
  el.dispatchEvent(new PointerEvent('pointerup', { bubbles: true, pointerId: 1 }));
  const fire = document.querySelector('[data-key="Space"]');
  fire.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, pointerId: 2 }));
})()`);
await sleep(1200);
await evaluate(`(() => {
  const fire = document.querySelector('[data-key="Space"]');
  fire.dispatchEvent(new PointerEvent('pointerup', { bubbles: true, pointerId: 2 }));
})()`);
await sleep(600);
await shot('touch-input');

const audio = await evaluate(`(() => {
  const ctx = window.__spyAudioContext;
  return ctx ? ctx.state : 'not-captured';
})()`);

await send('Browser.close').catch(() => {});
console.log(`ready=${ready} started=${started} audio=${audio} errors=${consoleErrors.length}`);
if (consoleErrors.length) console.log(consoleErrors.join('\n'));
