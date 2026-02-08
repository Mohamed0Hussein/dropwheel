'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';

export default function SideElements() {
  const [scrollProgress, setScrollProgress] = useState(0);

  useEffect(() => {
    const handleScroll = () => {
      const totalHeight = document.documentElement.scrollHeight - window.innerHeight;
      const progress = (window.scrollY / totalHeight) * 100;
      setScrollProgress(progress);
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <>
      <motion.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 1, duration: 0.6 }}
        className="fixed left-8 top-1/2 -translate-y-1/2 z-50 hidden lg:flex flex-col items-center gap-6"
      >
        <div className="h-24 w-[2px] relative overflow-hidden rounded-full" style={{ background: 'var(--border-primary)' }}>
          <div 
            className="absolute top-0 left-0 w-full transition-all duration-150 rounded-full"
            style={{ height: `${scrollProgress}%`, background: 'linear-gradient(180deg, var(--accent-gradient-from), var(--accent-gradient-to))' }}
          />
        </div>
        
        <div className="flex items-center gap-2">
          <span className="w-2 h-2 rounded-full animate-pulse" style={{ background: 'var(--accent-primary)' }} />
        </div>

        <a 
          href="mailto:mohamed.hussein0x1@gmail.com"
          className="vertical-text text-xs tracking-widest transition-colors"
          style={{ color: 'var(--text-tertiary)' }}
          onMouseEnter={(e) => e.currentTarget.style.color = 'var(--text-primary)'}
          onMouseLeave={(e) => e.currentTarget.style.color = 'var(--text-tertiary)'}
        >
          mohamed.hussein0x1@gmail.com
        </a>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 1, duration: 0.6 }}
        className="fixed right-8 bottom-8 z-50 hidden lg:block"
      >
        <div className="flex flex-col items-center gap-2 text-xs">
          <span className="vertical-text tracking-widest" style={{ color: 'var(--text-tertiary)' }}>SCROLL</span>
          <motion.div
            animate={{ y: [0, 8, 0] }}
            transition={{ duration: 1.5, repeat: Infinity }}
            className="w-px h-8"
            style={{ background: 'linear-gradient(to bottom, var(--text-primary), transparent)' }}
          />
        </div>
      </motion.div>
    </>
  );
}
