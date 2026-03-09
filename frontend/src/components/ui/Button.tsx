import React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
}

export const Button: React.FC<ButtonProps> = ({ className = '', variant = 'primary', ...props }) => {
    const variants = {
        primary: 'bg-[var(--accent-color)] text-black font-bold',
        secondary: 'bg-[#222] text-[var(--accent-color)] border border-[var(--accent-color)]',
        danger: 'bg-[#300] text-[#f00] border border-[#f00] hover:bg-[#f00] hover:text-black',
        ghost: 'bg-[#333] text-white hover:bg-[var(--accent-color)] hover:text-black',
    };

    return (
        <button
            className={`cursor-pointer text-[10px] h-5 px-1.5 flex items-center justify-center transition-colors disabled:opacity-50 ${variants[variant]} ${className}`}
            {...props}
        />
    );
};
