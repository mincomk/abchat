import React from 'react';

interface SettingsCardProps {
    title: string;
    children: React.ReactNode;
}

export const SettingsCard: React.FC<SettingsCardProps> = ({ title, children }) => {
    return (
        <section className="flex flex-col gap-2 max-w-[200px] w-full border border-[var(--border-color)] p-2 rounded bg-[var(--bg-color)]">
            <div className="flex justify-between items-center">
                <span className="text-[var(--accent-color)] font-bold text-[10px]">{title}</span>
            </div>
            {children}
        </section>
    );
};
