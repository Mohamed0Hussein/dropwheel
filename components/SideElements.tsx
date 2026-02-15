'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';

export default function SideElements() {
  const [scrollProgress, setScrollProgress] = useState(0);

  useEffect(() => {
    const handleScroll = () => {
      const totalHeight =
        document.documentElement.scrollHeight - window.innerHeight;
      const progress = totalHeight > 0 ? (window.scrollY / totalHeight) * 100 : 0;
      setScrollProgress(progress);
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <>
      {/* Left: Scroll Progress + Email */}
      <motion.div
        initial={{ opacity: 0, x: -16 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 1.2, duration: 0.5 }}
        className="fixed left-6 top-1/2 -translate-y-1/2 z-30 hidden xl:flex flex-col items-center gap-4"
      >
        <div
          className="h-20 w-[2px] relative overflow-hidden rounded-full"
          style={{ background: 'var(--border)' }}
        >
          <div
            className="absolute top-0 left-0 w-full transition-all duration-200 rounded-full"
            style={{
              height: `${scrollProgress}%`,
              background: 'linear-gradient(180deg, var(--gradient-from), var(--gradient-to))',
            }}
          />
        </div>
        <div className="flex items-center">
          <span className="status-dot" style={{ width: '6px', height: '6px' }} />
        </div>
        <a
          href="mailto:mohamed.hussein0x1@gmail.com"
          className="vertical-text text-[10px] tracking-[0.15em] font-medium transition-colors duration-200"
          style={{ color: 'var(--text-tertiary)' }}
          onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--accent)')}
          onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-tertiary)')}
        >
          mohamed.hussein0x1@gmail.com
        </a>
      </motion.div>
    </>
  );
}
