'use client';

import { motion, useMotionValue, useSpring } from 'framer-motion';
import { useEffect, useState } from 'react';

const HOVER_SELECTOR = 'a, button, input, textarea, select, [role="button"], .card, .skill-badge, .skill-pill';

export default function CustomCursor() {
  const [enabled, setEnabled] = useState(false);
  const [isHovering, setIsHovering] = useState(false);

  const dotX = useMotionValue(-100);
  const dotY = useMotionValue(-100);
  const ringX = useSpring(dotX, { stiffness: 350, damping: 30, mass: 0.5 });
  const ringY = useSpring(dotY, { stiffness: 350, damping: 30, mass: 0.5 });

  useEffect(() => {
    const fineHover = window.matchMedia('(hover: hover) and (pointer: fine)');
    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');

    const updateEnabled = () => setEnabled(fineHover.matches && !reducedMotion.matches);
    updateEnabled();
    fineHover.addEventListener('change', updateEnabled);
    reducedMotion.addEventListener('change', updateEnabled);

    return () => {
      fineHover.removeEventListener('change', updateEnabled);
      reducedMotion.removeEventListener('change', updateEnabled);
    };
  }, []);

  useEffect(() => {
    document.body.classList.toggle('custom-cursor-active', enabled);
    if (!enabled) return;

    const onMove = (e: MouseEvent) => {
      dotX.set(e.clientX);
      dotY.set(e.clientY);
    };
    const onOver = (e: MouseEvent) => {
      if ((e.target as Element)?.closest?.(HOVER_SELECTOR)) setIsHovering(true);
    };
    const onOut = (e: MouseEvent) => {
      if ((e.target as Element)?.closest?.(HOVER_SELECTOR)) setIsHovering(false);
    };

    window.addEventListener('mousemove', onMove, { passive: true });
    window.addEventListener('mouseover', onOver, { passive: true });
    window.addEventListener('mouseout', onOut, { passive: true });

    return () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseover', onOver);
      window.removeEventListener('mouseout', onOut);
      document.body.classList.remove('custom-cursor-active');
    };
  }, [enabled, dotX, dotY]);

  if (!enabled) return null;

  return (
    <>
      <motion.div
        className="custom-cursor-dot"
        style={{ x: dotX, y: dotY, translateX: '-50%', translateY: '-50%' }}
      />
      <motion.div
        className={`custom-cursor-ring${isHovering ? ' is-hovering' : ''}`}
        style={{ x: ringX, y: ringY, translateX: '-50%', translateY: '-50%' }}
      />
    </>
  );
}
