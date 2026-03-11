import React, { useState, useCallback, useRef, useEffect } from 'react';
import { LoginPage as Login, type LoginData } from './pages/LoginPage';
import { ChatPage as Chat } from './pages/ChatPage';
import { AdminPage as Admin } from './pages/AdminPage';
import { DBridgeClient, type User } from './api/dbridge-api';

const App: React.FC = () => {
    const [screen, setScreen] = useState<'login' | 'chat' | 'admin'>('login');
    const [user, setUser] = useState<User | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [windowSize, setWindowSize] = useState({ width: window.innerWidth, height: window.innerHeight });
    const [isDarkMode, setIsDarkMode] = useState(() => {
        const saved = localStorage.getItem('dbridge_theme');
        return saved ? saved === 'dark' : true;
    });
    const clientRef = useRef<DBridgeClient | null>(null);

    const toggleTheme = useCallback(() => {
        setIsDarkMode(prev => {
            const next = !prev;
            localStorage.setItem('dbridge_theme', next ? 'dark' : 'light');
            return next;
        });
    }, []);

    useEffect(() => {
        if (isDarkMode) {
            document.documentElement.classList.remove('light-mode');
        } else {
            document.documentElement.classList.add('light-mode');
        }
    }, [isDarkMode]);

    const handleLogin = useCallback(async (data: LoginData) => {
        const client = new DBridgeClient(data.channelId);
        clientRef.current = client;
        localStorage.setItem('dbridge_last_channel', data.channelId);
        setError(null);

        try {
            if (data.token && data.nickname) {
                // Token Login
                client.setCredentials(data.token, data.nickname, data.username, false);
                const newUser: User = {
                    username: data.username,
                    nickname: data.nickname,
                    is_admin: false
                };
                setUser(newUser);
            } else if (data.password) {
                // Password Login
                const { token, user: loggedUser } = await client.login(data.username, data.password);
                client.setCredentials(token, loggedUser.nickname, loggedUser.username, loggedUser.is_admin || false);
                setUser(loggedUser);
            } else {
                throw new Error('Missing password or token');
            }

            setScreen('chat');
        } catch (error: any) {
            setError(`${error.message}`);
        }
    }, []);

    const handleLogout = useCallback(() => {
        if (clientRef.current) {
            clientRef.current.clearCredentials();
            clientRef.current.disconnect();
            clientRef.current = null;
        }
        setUser(null);
        setScreen('login');
    }, []);

    useEffect(() => {
        const handleResize = () => {
            setWindowSize({ width: window.innerWidth, height: window.innerHeight });
        };
        window.addEventListener('resize', handleResize);

        // Auto-login from localStorage - Only if we don't already have a client
        if (!clientRef.current) {
            const lastChannel = localStorage.getItem('dbridge_last_channel') || 'main';
            const client = new DBridgeClient(lastChannel);
            const creds = client.getStoredCredentials();
            if (creds) {
                client.setCredentials(creds.token, creds.nickname, creds.username, creds.is_admin);
                clientRef.current = client;
                setUser({
                    username: creds.username,
                    nickname: creds.nickname,
                    is_admin: creds.is_admin
                });
                setScreen('chat');
            }
        }

        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const handleAdminClick = useCallback(() => {
        setScreen('admin');
    }, []);

    const handleBackToChat = useCallback(() => {
        setScreen('chat');
    }, []);

    return (
        <div className="w-full h-full relative overflow-hidden flex justify-center items-center bg-[var(--bg-color)]">
            {error && (
                <div className="fixed top-0 left-0 right-0 text-[var(--error-color)] text-[10px] text-center p-0.5 bg-black/80 z-[10000]">
                    {error}
                </div>
            )}
            {screen === 'login' && <Login onLogin={handleLogin} />}
            {screen === 'chat' && user && clientRef.current && (
                <Chat
                    client={clientRef.current}
                    username={user.username}
                    nickname={user.nickname}
                    isDarkMode={isDarkMode}
                    onToggleTheme={toggleTheme}
                    onAdmin={user.is_admin ? handleAdminClick : undefined}
                    onLogout={handleLogout}
                />
            )}
            {screen === 'admin' && user && clientRef.current && (
                <Admin
                    client={clientRef.current}
                    currentUsername={user.username}
                    onBack={handleBackToChat}
                />
            )}
            <div className="fixed bottom-0.5 right-0.5 text-[12px] text-[var(--secondary-text-color)] pointer-events-none z-[9999]">
                {windowSize.width}x{windowSize.height}
            </div>
        </div>
    );
}

export default App;
