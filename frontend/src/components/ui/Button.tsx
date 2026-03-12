import React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
}

export const Button: React.FC<ButtonProps> = ({ className = '', variant = 'primary', ...props }) => {
    const variants = {
        primary: 'bg-[var(--accent-color)] text-black font-bold',
        secondary: 'bg-[var(--button-secondary-bg)] text-[var(--accent-color)] border border-[var(--accent-color)]',
        danger: 'bg-[var(--error-bg)] text-[var(--error-color)] border border-[var(--error-color)] hover:bg-[var(--error-color)] hover:text-black',
        ghost: 'bg-[var(--button-ghost-bg)] text-[var(--text-color)] border border-[var(--border-color)] hover:bg-[var(--accent-color)] hover:text-black',
    };

    return (
        <button
            className={`cursor-pointer text-[10px] h-5 px-1.5 flex items-center justify-center transition-colors disabled:opacity-50 ${variants[variant]} ${className}`}
            {...props}
        />
    );
};
