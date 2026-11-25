import React from 'react';

export interface WatchMenuItem {
    label: string;
    onClick: () => void;
}

interface WatchMenuProps {
    title?: string;
    items: WatchMenuItem[];
    closeLabel: string;
    onClose: () => void;
}

/**
 * Full-screen action list for the watch layout. The header of a 396x396 screen
 * has no room for four labelled buttons, so every action lives here instead.
 */
export const WatchMenu: React.FC<WatchMenuProps> = ({ title, items, closeLabel, onClose }) => {
    return (
        <div className="fixed inset-0 z-50 bg-[var(--bg-color)] flex flex-col justify-center gap-1 p-2 items-center">
            {title && (
                <div className="text-center text-[11px] text-[var(--secondary-text-color)] mb-1 truncate">{title}</div>
            )}
            {items.map((item) => (
                <button
                    key={item.label}
                    type="button"
                    onClick={item.onClick}
                    className="watch-tap w-full max-w-[300px] border border-[var(--border-color)] bg-[var(--button-secondary-bg)] text-[var(--text-color)] px-2"
                >
                    {item.label}
                </button>
            ))}
            <button
                type="button"
                onClick={onClose}
                className="watch-tap w-full max-w-[300px] border border-[var(--accent-color)] text-[var(--accent-color)] mt-1 px-2"
            >
                {closeLabel}
            </button>
        </div>
    );
};
