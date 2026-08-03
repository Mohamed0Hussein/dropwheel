'use client';

import { motion, useMotionValue, useReducedMotion, useSpring } from 'framer-motion';
import { MouseEvent, ReactNode } from 'react';

/**
 * Wraps a button/link and nudges it toward the cursor within its bounds —
 * the one idiosyncratic hover signature reused across primary CTAs.
 * No-ops harmlessly on touch (no mousemove events fire there).
 */
export default function MagneticButton({
  children,
  strength = 0.35,
  className,
}: {
  children: ReactNode;
  strength?: number;
  className?: string;
}) {
  const prefersReducedMotion = useReducedMotion();
  const x = useMotionValue(0);
  const y = useMotionValue(0);
  const springX = useSpring(x, { stiffness: 200, damping: 15, mass: 0.4 });
  const springY = useSpring(y, { stiffness: 200, damping: 15, mass: 0.4 });

  function handleMouseMove(e: MouseEvent<HTMLDivElement>) {
    if (prefersReducedMotion) return;
    const { left, top, width, height } = e.currentTarget.getBoundingClientRect();
    x.set((e.clientX - left - width / 2) * strength);
    y.set((e.clientY - top - height / 2) * strength);
  }

  function handleMouseLeave() {
    x.set(0);
    y.set(0);
  }

  return (
    <motion.div
      className={`magnetic-btn inline-block${className ? ` ${className}` : ''}`}
      style={{ x: springX, y: springY }}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
    >
      {children}
    </motion.div>
  );
}
