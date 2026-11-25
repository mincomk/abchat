# Galaxy Watch (Wear OS) support

ABChat's frontend has a watch layout for round Wear OS displays, so the same web app
opens on a Samsung Galaxy Watch instead of squeezing the desktop UI into a 40mm circle.

## When the watch layout is used

`src/hooks/useWatchLayout.ts` matches the media query

```
(max-width: 460px) and (max-height: 460px)
```

A Galaxy Watch reports a *square* viewport: 396x396 (Watch 4/5/6/7 40mm), 450x450 or
480x480 (larger models, Watch 8). A phone is never square in portrait (360x800,
390x844), so the height bound is what keeps the watch layout off phones. A desktop
browser window resized to 396x396 also matches, which is how you test it without a watch.

## What changes in watch mode

| Area | Desktop | Watch |
| --- | --- | --- |
| Header | one row of `SETTINGS` / `ADMIN` / theme / `EXIT` buttons | `#channel` + a `☰` button |
| Actions | inline in the header | full-screen `WatchMenu` list |
| Messages | timestamp column, right-aligned `<nickname>`, text | `nickname` + text inline, no timestamp column |
| Composer | 26px input and button | 38px input and button |
| Settings | strip above the message list | full-screen scrollable panel |
| Login | one wrapping row of small fields | one field per line, 38px rows |
| Auto-focus | composer focused on mount | not focused (the on-screen keyboard would cover the screen) |

The classes used by the watch layout live in `frontend/src/index.css`:

- `.watch-tap` raises any control to a 38px minimum tap target and 13px text.
- `.watch-inset` keeps the top and bottom rows clear of the bezel; on round displays
  that report the CSS `shape` media feature the inset grows to 20px.

## Opening it on the watch

Wear OS does not ship a web browser and does not install PWAs, so there are two limits
up front: this is the web app rendered small, not a native watch app, and you need a
browser app on the watch to reach it.

1. Install a Wear OS browser app from the Play Store on the watch (they are all
   third-party; none of them has been verified against this layout on physical hardware).
2. Open `https://abc.allen.sh` in it.
3. Log in. Password mode works but typing on a watch keyboard is slow; token mode
   (`TK`) needs a token, username and nickname, and then it stays logged in on that device.

Web pushes are only sent to browsers that expose `PushManager`; whether the watch browser
does is untested. With the tab open, messages arrive over the existing websocket.

## Testing the layout without a watch

```
cd frontend
pnpm dev
```

Then open <http://localhost:5173/watch-preview.html> and resize the window to
396x396 (or use the browser's device toolbar with a custom 396x396 viewport). The
preview page renders `ChatPage` against a fake client with sample history and does not
touch the backend. It is a Vite dev entry only (`watch-preview.html` at the frontend
root), so it is not part of the production build.
