import React, { useState, useCallback, useRef, useEffect } from 'react';
import { Routes, Route, useNavigate, useLocation, Navigate } from 'react-router';
import { LoginPage as Login, type LoginData } from './pages/LoginPage';
import { ChatPage as Chat } from './pages/ChatPage';
import { AdminPage as Admin } from './pages/AdminPage';
import { DBridgeClient, type User } from './api/dbridge-api';

const App: React.FC = () => {
    const navigate = useNavigate();
    const location = useLocation();
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

            // Redirect based on current location
            if (location.pathname === '/admin' && user?.is_admin) {
                // Stay on admin
            } else {
                navigate('/');
            }
        } catch (error: any) {
            setError(`${error.message}`);
        }
    }, [location.pathname, navigate]);

    const handleLogout = useCallback(() => {
        if (clientRef.current) {
            clientRef.current.clearCredentials();
            clientRef.current.disconnect();
            clientRef.current = null;
        }
        setUser(null);
        navigate('/');
    }, [navigate]);

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
            }
        }

        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const handleAdminClick = useCallback(() => {
        navigate('/admin');
    }, [navigate]);

    const handleBackToChat = useCallback(() => {
        navigate('/');
    }, [navigate]);

    useEffect(() => {
        if (location.pathname === '/admin' && user && !user.is_admin) {
            navigate('/');
        }
    }, [location.pathname, user, navigate]);

    return (
        <div className="w-full h-full relative overflow-hidden flex justify-center items-center bg-[var(--bg-color)]">
            {error && (
                <div className="fixed top-0 left-0 right-0 text-[var(--error-color)] text-[10px] text-center p-0.5 bg-black/80 z-[10000]">
                    {error}
                </div>
            )}
            {(!user || !clientRef.current) ? (
                <Login onLogin={handleLogin} />
            ) : (
                <Routes>
                    <Route
                        path="/"
                        element={
                            <Chat
                                client={clientRef.current}
                                username={user.username}
                                nickname={user.nickname}
                                isDarkMode={isDarkMode}
                                onToggleTheme={toggleTheme}
                                onAdmin={user.is_admin ? handleAdminClick : undefined}
                                onLogout={handleLogout}
                            />
                        }
                    />
                    <Route
                        path="/admin"
                        element={
                            user.is_admin ? (
                                <Admin
                                    client={clientRef.current}
                                    currentUsername={user.username}
                                    onBack={handleBackToChat}
                                />
                            ) : (
                                <Navigate to="/" />
                            )
                        }
                    />
                </Routes>
            )}
            <div className="fixed bottom-0.5 right-0.5 text-[12px] text-[var(--secondary-text-color)] pointer-events-none z-[9999]">
                {windowSize.width}x{windowSize.height}
            </div>
        </div>
    );
}

export default App;
