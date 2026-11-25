/* eslint-disable react-refresh/only-export-components -- dev-only preview entry, never bundled */
import React from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import './i18n/config';
import { ChatPage } from './pages/ChatPage';
import { LoginPage } from './pages/LoginPage';
import { WatchMenu } from './components/chat/WatchMenu';
import { SettingsOverlay } from './components/chat/settings/SettingsOverlay';
import type { DBridgeClient } from './api/dbridge-api';

/**
 * Dev-only harness for the watch layout (see docs/WATCH.md).
 * Open /watch-preview.html with a 396x396 viewport.
 * `?view=login|menu|settings` switches to the other watch screens.
 */
const now = Math.floor(Date.now() / 1000);

const history = [
    { id: '1', content: 'wake up, the drone test is at 9', timestamp: now - 720, sender: { username: 'mincomk', nickname: 'Minco' } },
    { id: '2', content: 'on my way, ETA 10 min', timestamp: now - 660, sender: { username: 'allen', nickname: 'Allen' } },
    { id: '3', content: 'bring the ST-Link and the spare props', timestamp: now - 300, sender: { username: 'mincomk', nickname: 'Minco' } },
    { id: '4', content: 'roger. also the battery is charged', timestamp: now - 120, sender: { username: 'allen', nickname: 'Allen' } },
];

const fakeClient = {
    channelId: 'main',
    onMessage: () => { },
    onOpen: () => { },
    onClose: () => { },
    onError: () => { },
    connect: () => { },
    disconnect: () => { },
    sendMessage: () => { },
    getHistoricalMessages: async () => history,
} as unknown as DBridgeClient;

const view = new URLSearchParams(window.location.search).get('view') ?? 'chat';

const Preview: React.FC = () => {
    switch (view) {
        case 'login':
            return <LoginPage onLogin={() => { }} />;
        case 'menu':
            return (
                <WatchMenu
                    title="#main @mincomk"
                    items={[
                        { label: 'SETTINGS', onClick: () => { } },
                        { label: 'LIGHT', onClick: () => { } },
                        { label: 'ADMIN', onClick: () => { } },
                        { label: 'EXIT', onClick: () => { } },
                    ]}
                    closeLabel="CLOSE"
                    onClose={() => { }}
                />
            );
        case 'settings':
            return (
                <SettingsOverlay
                    client={fakeClient}
                    onClose={() => { }}
                    username="mincomk"
                    watch
                />
            );
        default:
            return (
                <ChatPage
                    client={fakeClient}
                    username="mincomk"
                    nickname="Minco"
                    isDarkMode
                    onToggleTheme={() => { }}
                    onAdmin={() => { }}
                    onLogout={() => { }}
                />
            );
    }
};

createRoot(document.getElementById('root')!).render(<Preview />);
