import React, { useState, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient, type Message } from '../api/dbridge-api';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { MessageRow, type ChatMessage } from '../components/chat/MessageRow';
import { SettingsOverlay } from '../components/chat/settings/SettingsOverlay';

interface ChatProps {
    client: DBridgeClient;
    username: string;
    nickname: string;
    isDarkMode: boolean;
    onToggleTheme: () => void;
    onAdmin?: () => void;
    onLogout?: () => void;
}

export const ChatPage: React.FC<ChatProps> = ({ client, username, nickname, isDarkMode, onToggleTheme, onAdmin, onLogout }) => {
    const { t } = useTranslation();
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [inputValue, setInputValue] = useState('');
    const [showSettings, setShowSettings] = useState(false);

    const messageListRef = useRef<HTMLDivElement>(null);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
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
        const addSystemMessage = (content: string, systemType: 'info' | 'error' = 'info') => {
            setMessages(prev => [...prev, {
                id: Math.random().toString(36).substring(7),
                type: 'system',
                systemType,
                content,
                timestamp: Math.floor(Date.now() / 1000)
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

        addSystemMessage(t('chat.system.joining', { nickname, username }), 'info');

        let isMounted = true;

        const loadHistory = async () => {
            try {
                addSystemMessage(t('chat.system.loading'), 'info');
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
                    addSystemMessage(t('chat.system.loaded', { count: history.length }), 'info');
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

        setTimeout(() => inputRef.current?.focus(), 100);

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

    return (
        <div className="w-full h-full grid grid-rows-[25px_auto_1fr_35px] pb-safe">
            <div className="flex justify-between items-center px-2.5 bg-[var(--header-bg)] border-b border-[var(--border-color)] text-[11px]">
                <span className="text-[var(--accent-color)] font-bold">#{client.channelId}</span>
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
            {showSettings && (
                <SettingsOverlay 
                    client={client} 
                    onClose={() => setShowSettings(false)} 
                />
            )}
            <div className="overflow-y-auto p-1.5 flex flex-col gap-0.5 scrollbar-thin scrollbar-thumb-[var(--border-color)] scrollbar-track-transparent" ref={messageListRef}>
                {messages.map((msg) => (
                    <MessageRow key={msg.id} msg={msg} />
                ))}
            </div>
            <form className="h-[35px] bg-[var(--bg-color)] border-t border-[var(--border-color)] flex p-[2px_5px] gap-1.5 items-center" onSubmit={handleSubmit}>
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
        </div>
    );
};
