'use client';

import { motion } from 'framer-motion';
import { X, ExternalLink, Github, Layers, AlertTriangle, Sparkles, GraduationCap } from 'lucide-react';
import Link from 'next/link';
import { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import type { Project } from './Projects';

const FOCUSABLE = 'a[href], button:not([disabled]), input, textarea, select, [tabindex]:not([tabindex="-1"])';

export default function ProjectDetailModal({
  project,
  onClose,
}: {
  project: Project;
  onClose: () => void;
}) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    closeButtonRef.current?.focus();

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
        return;
      }
      if (e.key !== 'Tab' || !dialogRef.current) return;

      const focusables = Array.from(dialogRef.current.querySelectorAll<HTMLElement>(FOCUSABLE));
      if (focusables.length === 0) return;
      const first = focusables[0];
      const last = focusables[focusables.length - 1];

      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      document.body.style.overflow = previousOverflow;
    };
  }, [onClose]);

  // Portal to <body>: the Projects section can be 3D-transformed by ScrollTilt,
  // and a transformed ancestor would trap this `position: fixed` overlay
  // inside the section instead of covering the viewport.
  return createPortal(
    <motion.div
      className="fixed inset-0 z-[70] flex items-center justify-center p-4 md:p-8"
      style={{ background: 'rgba(2, 3, 8, 0.72)', backdropFilter: 'blur(8px)', WebkitBackdropFilter: 'blur(8px)' }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.25 }}
      onClick={onClose}
    >
      <motion.div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="project-modal-title"
        className="relative w-full max-w-3xl max-h-[85vh] overflow-y-auto rounded-[24px]"
        style={{
          background: 'var(--card-bg)',
          border: '1px solid var(--card-border)',
          boxShadow: '0 40px 100px -20px rgba(0,0,0,0.6)',
        }}
        initial={{ opacity: 0, y: 32, scale: 0.96 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 24, scale: 0.97 }}
        transition={{ duration: 0.35, ease: [0.25, 1, 0.5, 1] }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Close button */}
        <button
          ref={closeButtonRef}
          type="button"
          onClick={onClose}
          aria-label="Close project details"
          className="absolute top-5 right-5 z-20 flex items-center justify-center w-9 h-9 rounded-full transition-colors duration-300"
          style={{ background: 'var(--surface-elevated)', border: '1px solid var(--border-strong)', color: 'var(--text-secondary)' }}
        >
          <X className="w-4 h-4" />
        </button>

        {/* Cover / gallery hero */}
        <div className="relative h-56 md:h-64 overflow-hidden" style={{ background: project.pattern, backgroundColor: 'rgba(0,0,0,0.1)' }}>
          <div className="absolute top-0 left-0 right-0 h-1" style={{ background: project.gradient }} />
          <div className="absolute inset-0 flex items-end p-6 md:p-8">
            <div>
              <span
                className="text-[0.65rem] font-bold uppercase tracking-[0.24em] block mb-2"
                style={{ color: 'rgba(255,255,255,0.8)' }}
              >
                {project.subtitle}
              </span>
              <h2 id="project-modal-title" className="text-3xl md:text-4xl font-display font-black text-white tracking-tight">
                {project.title}
              </h2>
            </div>
          </div>
        </div>

        {/* Mini gallery strip — generated abstract tiles (no live screenshots captured) */}
        <div className="grid grid-cols-3 gap-2 p-4" aria-hidden="true">
          {[0, 1, 2].map((i) => (
            <div
              key={i}
              className="h-16 md:h-20 rounded-xl"
              style={{ background: project.pattern, backgroundColor: 'rgba(0,0,0,0.06)', opacity: 0.5 + i * 0.15 }}
            />
          ))}
        </div>

        <div className="p-6 md:p-8 pt-2 space-y-8">
          {/* Description + tech + links */}
          <div>
            <p className="text-base leading-relaxed mb-5" style={{ color: 'var(--text-secondary)' }}>
              {project.description}
            </p>
            <div className="flex flex-wrap gap-2 mb-5">
              {project.tech.map((t) => (
                <span
                  key={t}
                  className="text-[0.7rem] font-semibold px-2.5 py-1 rounded-lg border"
                  style={{
                    background: 'var(--surface-elevated)',
                    borderColor: 'var(--border-strong)',
                    color: 'var(--text-secondary)',
                    fontFamily: 'JetBrains Mono, var(--font-mono), monospace',
                  }}
                >
                  {t}
                </span>
              ))}
            </div>
            <div className="flex items-center gap-3">
              {project.live && (
                <Link href={project.live} target="_blank" rel="noopener noreferrer" className="btn-primary py-2! px-5! text-sm! rounded-xl gap-1.5">
                  <ExternalLink className="w-3.5 h-3.5" />
                  Live Demo
                </Link>
              )}
              {project.github && (
                <Link href={project.github} target="_blank" rel="noopener noreferrer" className="btn-outline py-2! px-5! text-sm! rounded-xl gap-1.5">
                  <Github className="w-3.5 h-3.5" />
                  Source
                </Link>
              )}
            </div>
          </div>

          {/* Architecture */}
          <DetailSection icon={Layers} title="Architecture" color={project.accentColor}>
            {project.architecture}
          </DetailSection>

          {/* Challenges */}
          <DetailSection icon={AlertTriangle} title="Challenges" color={project.accentColor}>
            {project.challenges}
          </DetailSection>

          {/* Key Features */}
          <div>
            <SectionHeading icon={Sparkles} title="Key Features" color={project.accentColor} />
            <ul className="space-y-2.5 mt-3">
              {project.features.map((f) => (
                <li key={f} className="flex items-start gap-2.5 text-sm leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
                  <span className="mt-1.5 w-1.5 h-1.5 rounded-full shrink-0" style={{ background: project.accentColor }} />
                  {f}
                </li>
              ))}
            </ul>
          </div>

          {/* Lessons Learned */}
          <DetailSection icon={GraduationCap} title="Lessons Learned" color={project.accentColor}>
            {project.lessonsLearned}
          </DetailSection>
        </div>
      </motion.div>
    </motion.div>,
    document.body
  );
}

function SectionHeading({ icon: Icon, title, color }: { icon: typeof Layers; title: string; color: string }) {
  return (
    <div className="flex items-center gap-2.5">
      <Icon className="w-4 h-4" style={{ color }} />
      <h3 className="text-sm font-bold uppercase tracking-widest" style={{ color: 'var(--text-primary)' }}>
        {title}
      </h3>
    </div>
  );
}

function DetailSection({
  icon,
  title,
  color,
  children,
}: {
  icon: typeof Layers;
  title: string;
  color: string;
  children: string;
}) {
  return (
    <div>
      <SectionHeading icon={icon} title={title} color={color} />
      <p className="text-sm leading-relaxed mt-3" style={{ color: 'var(--text-secondary)' }}>
        {children}
      </p>
    </div>
  );
}
