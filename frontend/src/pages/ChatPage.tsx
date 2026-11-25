import React, { useState, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient, type Message, type User } from '../api/dbridge-api';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { MessageRow, type ChatMessage } from '../components/chat/MessageRow';
import { SettingsOverlay } from '../components/chat/settings/SettingsOverlay';
import { WatchMenu } from '../components/chat/WatchMenu';
import { useWatchLayout, WATCH_MEDIA_QUERY } from '../hooks/useWatchLayout';

interface ChatProps {
    client: DBridgeClient;
    username: string;
    nickname: string;
    isDarkMode: boolean;
    onToggleTheme: () => void;
    onAdmin?: () => void;
    onLogout?: () => void;
    onUpdateUser?: (user: User) => void;
}

export const ChatPage: React.FC<ChatProps> = ({ client, username, nickname, isDarkMode, onToggleTheme, onAdmin, onLogout, onUpdateUser }) => {
    const { t } = useTranslation();
    const isWatch = useWatchLayout();
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [inputValue, setInputValue] = useState('');
    const [showSettings, setShowSettings] = useState(false);
    const [showMenu, setShowMenu] = useState(false);

    const messageListRef = useRef<HTMLDivElement>(null);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                if (messageListRef.current) {
                    messageListRef.current.scrollTop = messageListRef.current.scrollHeight;
                }
                if (document.activeElement instanceof HTMLElement) {
                    document.activeElement.blur();
                }
                return;
            }

            // Ignore if any input/textarea is already focused
            if (document.activeElement?.tagName === 'INPUT' || document.activeElement?.tagName === 'TEXTAREA') {
                return;
            }

            if ((e.altKey && e.key.toLowerCase() === 'i') || e.key === '/') {
                e.preventDefault();
                inputRef.current?.focus();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, []);

    useEffect(() => {
        const addSystemMessage = (content: string, systemType: 'info' | 'error' = 'info', quiet = false) => {
            setMessages(prev => [...prev, {
                id: Math.random().toString(36).substring(7),
                type: 'system',
                systemType,
                content,
                timestamp: Math.floor(Date.now() / 1000),
                quiet
            }]);
        };

        const handleMessage = (msg: Message) => {
            setMessages(prev => [...prev, {
                id: msg.id,
                type: 'user',
                sender: msg.sender,
                content: msg.content,
                timestamp: msg.timestamp
            }]);
        };

        client.onMessage(handleMessage);
        client.onOpen(() => addSystemMessage(t('chat.system.connected'), 'info'));
        client.onClose(() => addSystemMessage(t('chat.system.lost'), 'error'));
        client.onError((err) => addSystemMessage(err, 'error'));

        // Progress chatter is marked quiet and filtered out when rendering on a
        // watch, where three status lines would eat half of a 396px screen.
        addSystemMessage(t('chat.system.joining', { nickname, username }), 'info', true);

        let isMounted = true;

        const loadHistory = async () => {
            try {
                addSystemMessage(t('chat.system.loading'), 'info', true);
                const history = await client.getHistoricalMessages(50);

                if (!isMounted) return;

                const sortedHistory: ChatMessage[] = history
                    .sort((a, b) => a.timestamp - b.timestamp)
                    .map(msg => ({
                        id: msg.id,
                        type: 'user',
                        sender: msg.sender,
                        content: msg.content,
                        timestamp: msg.timestamp
                    }));

                setMessages(prev => {
                    // Filter out history messages that might already be in the list
                    const existingIds = new Set(prev.map(m => m.id));
                    const newHistory = sortedHistory.filter(m => !existingIds.has(m.id));
                    return [...newHistory, ...prev].sort((a, b) => a.timestamp - b.timestamp);
                });

                if (history.length > 0) {
                    addSystemMessage(t('chat.system.loaded', { count: history.length }), 'info', true);
                } else {
                    addSystemMessage(t('chat.system.no_history'), 'info');
                }
            } catch (err) {
                if (!isMounted) return;
                addSystemMessage(t('chat.system.failed_history', { error: err instanceof Error ? err.message : String(err) }), 'error');
            }
        };

        loadHistory();
        client.connect();

        // On a watch the on-screen keyboard covers most of the display, so only
        // focus the composer when the user taps it.
        if (!window.matchMedia(WATCH_MEDIA_QUERY).matches) {
            setTimeout(() => inputRef.current?.focus(), 100);
        }

        return () => {
            isMounted = false;
            // Clear callbacks to avoid updates to an unmounted component
            client.onMessage(() => { });
            client.onOpen(() => { });
            client.onClose(() => { });
            client.onError(() => { });
            client.disconnect();
            setMessages([]); // Reset messages on cleanup to avoid duplication when re-mounting (Strict Mode)
        };
    }, [client, username, nickname, t]);

    useEffect(() => {
        if (messageListRef.current) {
            messageListRef.current.scrollTop = messageListRef.current.scrollHeight;
        }
    }, [messages]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (inputValue.trim()) {
            const content = inputValue.trim();

            client.sendMessage(content);
            setInputValue('');
        }
    };

    const visibleMessages = isWatch ? messages.filter(msg => !msg.quiet) : messages;

    const watchMenuItems = [
        { label: t('chat.settings'), onClick: () => { setShowMenu(false); setShowSettings(true); } },
        { label: isDarkMode ? 'LIGHT' : 'DARK', onClick: () => { setShowMenu(false); onToggleTheme(); } },
        ...(onAdmin ? [{ label: t('chat.admin'), onClick: () => { setShowMenu(false); onAdmin(); } }] : []),
        ...(onLogout ? [{ label: t('chat.exit'), onClick: () => { setShowMenu(false); onLogout(); } }] : []),
    ];

    return (
        <div className="relative w-full h-full flex flex-col pb-safe overflow-hidden" data-watch={isWatch ? 'true' : 'false'}>
            {isWatch ? (
                <div className="watch-inset flex-none flex items-center justify-center gap-1.5 pt-3 pb-0.5">
                    <span className="text-[13px] text-[var(--accent-color)] font-bold truncate min-w-0">
                        #{client.channelId}
                    </span>
                    <button
                        type="button"
                        onClick={() => setShowMenu(true)}
                        aria-label={t('chat.settings')}
                        title={t('chat.settings')}
                        className="watch-tap flex-none border border-[var(--border-color)] bg-[var(--button-ghost-bg)] text-[var(--text-color)] px-2 leading-none"
                    >
                        &#9776;
                    </button>
                </div>
            ) : (
                <div className="flex-none h-[25px] flex justify-between items-center px-2.5 bg-[var(--header-bg)] border-b border-[var(--border-color)] text-[11px]">
                    <div className="flex gap-2 items-center">
                        <span className="text-[var(--accent-color)] font-bold">#{client.channelId}</span>
                        <span className="text-[var(--secondary-text-color)] opacity-70">@{username}</span>
                    </div>
                    <div className="flex gap-1.5">
                        <Button variant="ghost" className="!h-4 !px-1 !text-[9px]" onClick={() => setShowSettings(!showSettings)}>
                            {t('chat.settings')}
                        </Button>
                        {onAdmin && <Button variant="ghost" className="!h-4 !px-1 !text-[9px]" onClick={onAdmin}>{t('chat.admin')}</Button>}
                        <Button variant="ghost" className="!h-4 !px-1 !text-[9px]" onClick={onToggleTheme}>
                            {isDarkMode ? 'DARK' : 'LIGHT'}
                        </Button>
                        {onLogout && <Button variant="ghost" className="!h-4 !px-1 !text-[9px]" onClick={onLogout}>{t('chat.exit')}</Button>}
                    </div>
                </div>
            )}
            {isWatch && showMenu && (
                <WatchMenu
                    title={`#${client.channelId} @${username}`}
                    items={watchMenuItems}
                    closeLabel={t('change_pwd.close')}
                    onClose={() => setShowMenu(false)}
                />
            )}
            {showSettings && (
                <SettingsOverlay
                    client={client}
                    onClose={() => setShowSettings(false)}
                    onUpdateUser={onUpdateUser}
                    username={username}
                    watch={isWatch}
                />
            )}
            <div
                className={`flex-1 min-h-0 overflow-y-auto flex flex-col mx-auto w-full scrollbar-thin scrollbar-thumb-[var(--border-color)] scrollbar-track-transparent ${isWatch ? 'max-w-[300px] px-1.5 pt-6 pb-2 gap-1' : 'p-1.5 gap-0.5'}`}
                ref={messageListRef}
            >
                {visibleMessages.map((msg) => (
                    <MessageRow key={msg.id} msg={msg} compact={isWatch} />
                ))}
            </div>
            {isWatch ? (
                <div className="flex-none flex justify-center">
                    <form className="w-full max-w-[270px] flex items-center gap-1 px-1 pt-1 pb-14" onSubmit={handleSubmit}>
                        <Input
                            type="text"
                            placeholder={t('chat.placeholder')}
                            autoComplete="off"
                            className="flex-1 watch-tap min-w-0"
                            value={inputValue}
                            onChange={(e) => setInputValue(e.target.value)}
                            ref={inputRef}
                        />
                        <Button type="submit" variant="ghost" className="watch-tap flex-none !px-2">
                            {t('chat.send')}
                        </Button>
                    </form>
                </div>
            ) : (
                <form className="flex-none h-[35px] bg-[var(--bg-color)] border-t border-[var(--border-color)] flex p-[2px_5px] gap-1.5 items-center" onSubmit={handleSubmit}>
                    <Input
                        type="text"
                        placeholder={t('chat.placeholder')}
                        autoComplete="off"
                        className="flex-1 !h-[26px]"
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        ref={inputRef}
                    />
                    <Button type="submit" variant="ghost" className="!h-[26px] !px-2.5">{t('chat.send')}</Button>
                </form>
            )}
        </div>
    );
};
