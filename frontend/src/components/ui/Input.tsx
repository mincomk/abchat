import React, { forwardRef } from 'react';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}

export const Input = forwardRef<HTMLInputElement, InputProps>(({ className = '', ...props }, ref) => {
    return (
        <input
            ref={ref}
            className={`bg-[var(--input-bg)] border border-[var(--border-color)] text-[var(--text-color)] px-1 py-0.5 outline-none text-[10px] h-5 focus:border-[var(--accent-color)] ${className}`}
            {...props}
        />
    );
});

Input.displayName = 'Input';
