import { useSyncExternalStore } from 'react';

/**
 * Round Wear OS screens (Galaxy Watch 4/5/6/7/8, Pixel Watch, ...) report a
 * square viewport: 396x396 on a 40mm Galaxy Watch, 450x450 or 480x480 on the
 * bigger ones. A phone is never square in portrait (390x844, 360x800), so a
 * bounded *height* is what keeps watch mode off phones while still letting a
 * 396x396 desktop window preview it.
 */
export const WATCH_MEDIA_QUERY = '(max-width: 460px) and (max-height: 460px)';

const supported = (): boolean =>
    typeof window !== 'undefined' && typeof window.matchMedia === 'function';

const subscribe = (onChange: () => void): (() => void) => {
    if (!supported()) return () => { };
    const mql = window.matchMedia(WATCH_MEDIA_QUERY);
    mql.addEventListener('change', onChange);
    return () => mql.removeEventListener('change', onChange);
};

const getSnapshot = (): boolean => supported() && window.matchMedia(WATCH_MEDIA_QUERY).matches;

/** True when the viewport is a small, square-ish screen, i.e. a watch. */
export const useWatchLayout = (): boolean => useSyncExternalStore(subscribe, getSnapshot, () => false);
