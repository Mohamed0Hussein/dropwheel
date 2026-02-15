'use client';

import { Moon, Sun } from 'lucide-react';
import { motion } from 'framer-motion';
import { useEffect, useState } from 'react';

export default function ThemeToggle() {
  const [mounted, setMounted] = useState(false);
  const [currentTheme, setCurrentTheme] = useState<'dark' | 'light'>('light');

  useEffect(() => {
    setMounted(true);
    const savedTheme = (localStorage.getItem('theme') as 'dark' | 'light') || 
      (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    setCurrentTheme(savedTheme);
    document.documentElement.setAttribute('data-theme', savedTheme);
  }, []);

  const handleToggle = () => {
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setCurrentTheme(newTheme);
    localStorage.setItem('theme', newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
  };

  if (!mounted) {
    return (
      <div
        className="theme-toggle"
        style={{ width: '44px', height: '44px', padding: '10px' }}
      >
        <div style={{ width: '20px', height: '20px' }} />
      </div>
    );
  }

  return (
    <motion.button
      onClick={handleToggle}
      className="theme-toggle group"
      aria-label={`Switch to ${currentTheme === 'dark' ? 'light' : 'dark'} mode`}
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
    >
      <motion.div
        className="theme-toggle-inner"
        initial={false}
        animate={{ rotate: currentTheme === 'dark' ? 0 : 180 }}
        transition={{ duration: 0.5, ease: 'easeInOut' }}
      >
        {currentTheme === 'dark' ? (
          <Moon className="w-[18px] h-[18px]" />
        ) : (
          <Sun className="w-[18px] h-[18px]" />
        )}
      </motion.div>
      <span className="theme-toggle-bg" />
    </motion.button>
  );
}
