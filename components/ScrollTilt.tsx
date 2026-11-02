'use client';

/**
 * ScrollTilt — folds a page section in 3D as it scrolls through the viewport.
 *
 * Each section is hinged at its edges:
 *  - Entering from below, it is hinged at its TOP edge and starts folded back
 *    into the screen, then swings up flat as its top edge rises.
 *  - Leaving at the top, it is hinged at its BOTTOM edge and folds away.
 *
 * Because every section hinges on the seam it shares with its neighbour, the
 * page stays visually continuous — it reads like a sheet rolling over a drum —
 * and a tilted section only ever recedes (never pokes out toward the camera),
 * so very tall sections are safe.
 *
 * When flat, the wrapper's transform is `none`, so text stays crisp and the
 * wrapper does not become a containing block or stacking context. Users with
 * `prefers-reduced-motion` get no tilt at all.
 *
 * The tilt is switched on only after mount, through a motion value. Server
 * HTML and the hydration render are therefore always flat — React keeps the
 * server's style attribute on a hydration mismatch, so deciding this during
 * render would leave reduced-motion users stuck with the server's tilt.
 */

import { motion, useMotionValue, useScroll, useTransform } from 'framer-motion';
import { useEffect, useRef, type ReactNode } from 'react';

/** Fold angle (deg) when a section's edge is at the very edge of the viewport. */
const MAX_TILT = 32;
/** Camera distance (px). Smaller = more dramatic foreshortening. */
const PERSPECTIVE = 1100;

interface ScrollTiltProps {
  children: ReactNode;
  /** Fold in while entering from the bottom of the viewport. */
  tiltIn?: boolean;
  /** Fold away while leaving through the top of the viewport. */
  tiltOut?: boolean;
}

export default function ScrollTilt({ children, tiltIn = true, tiltOut = true }: ScrollTiltProps) {
  const ref = useRef<HTMLDivElement>(null);
  /** 1 once mounted on a client that allows motion, 0 otherwise. */
  const enabled = useMotionValue(0);

  useEffect(() => {
    const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
    const sync = () => enabled.set(mq.matches ? 0 : 1);
    sync();
    mq.addEventListener('change', sync);
    return () => mq.removeEventListener('change', sync);
  }, [enabled]);

  // 0 → 1 while the top edge travels from the viewport bottom up to 35% height.
  const { scrollYProgress: enter } = useScroll({
    target: ref,
    offset: ['start end', 'start 0.35'],
  });
  // 0 → 1 while the bottom edge travels from 65% height up to the viewport top.
  const { scrollYProgress: exit } = useScroll({
    target: ref,
    offset: ['end 0.65', 'end start'],
  });

  // Negative rotateX tips content below the hinge away from the viewer;
  // positive tips content above the hinge away. The two ranges never overlap.
  const transform = useTransform([enter, exit, enabled], ([i, o, on]: number[]) => {
    const angle = (tiltIn ? -(1 - i) * MAX_TILT : 0) + (tiltOut ? o * MAX_TILT : 0);
    return on && angle !== 0
      ? `perspective(${PERSPECTIVE}px) rotateX(${angle.toFixed(3)}deg)`
      : 'none';
  });
  // Hinge on the edge that is inside the viewport. Switching is seamless
  // because the section is flat whenever neither phase is active.
  const transformOrigin = useTransform(exit, (o) => (o > 0 ? '50% 100%' : '50% 0%'));

  return (
    <motion.div ref={ref} style={{ transform, transformOrigin }}>
      {children}
    </motion.div>
  );
}
