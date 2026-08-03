'use client';

import { motion, Variants } from 'framer-motion';
import { Code2, Server, Database as DatabaseIcon, Workflow } from 'lucide-react';
import SkillIcon from './SkillIcon';

/* ── Data ── */
const row1 = [
  'JavaScript (ES6+)', 'TypeScript', 'React 18', 'Next.js 15', 'Redux Toolkit',
  'Tailwind CSS v4', 'Framer Motion', 'Radix UI', 'Material UI', 'CSS3 / Sass', 'HTML5',
];

const row2 = [
  'Node.js', 'Express.js', 'MongoDB', 'Mongoose', 'PostgreSQL', 'Prisma ORM',
  'REST APIs', 'GraphQL', 'Git', 'GitHub Actions', 'Docker', 'Vercel', 'Vite', 'Jest', 'Figma',
];

const stackCategories = [
  {
    title: 'Frontend Architecture',
    description: 'Robust, scalable, pixel-perfect user interfaces.',
    number: '01',
    categoryIcon: Code2,
    gradient: 'linear-gradient(135deg, #7c3aed, #3b82f6)',
    accentColor: '#7c3aed',
    skills: ['JavaScript (ES6+)', 'TypeScript', 'React 18', 'Next.js 14/15', 'Redux Toolkit', 'Tailwind CSS v4', 'Material UI', 'Framer Motion', 'Radix UI', 'CSS3/Sass'],
  },
  {
    title: 'Backend Integration',
    description: 'Resilient server-side logic and scalable APIs.',
    number: '02',
    categoryIcon: Server,
    gradient: 'linear-gradient(135deg, #3b82f6, #0ea5e9)',
    accentColor: '#3b82f6',
    skills: ['Node.js', 'Express.js', 'REST APIs', 'Next.js API Routes', 'GraphQL Basics'],
  },
  {
    title: 'Database Management',
    description: 'Structured data modelling and high-performance persistence.',
    number: '03',
    categoryIcon: DatabaseIcon,
    gradient: 'linear-gradient(135deg, #0891b2, #22d3ee)',
    accentColor: '#0891b2',
    skills: ['MongoDB', 'Mongoose', 'PostgreSQL', 'Prisma ORM'],
  },
  {
    title: 'DevOps & Tooling',
    description: 'Modern workflow optimisation, CI/CD, and deployments.',
    number: '04',
    categoryIcon: Workflow,
    gradient: 'linear-gradient(135deg, #0d9488, #14b8a6)',
    accentColor: '#0d9488',
    skills: ['Git', 'GitHub Actions', 'Docker', 'Vercel', 'Vite', 'Postman', 'Figma', 'Jest'],
  },
];

/* All skills, deduplicated, for the centered bubble cloud */
const allSkills = Array.from(new Set([...row1, ...row2]));

const bubbleColors = [
  { bg: 'rgba(124,58,237,0.08)', border: 'rgba(124,58,237,0.2)' },
  { bg: 'rgba(59,130,246,0.08)', border: 'rgba(59,130,246,0.2)' },
  { bg: 'rgba(34,211,238,0.08)', border: 'rgba(34,211,238,0.2)' },
  { bg: 'rgba(20,184,166,0.08)', border: 'rgba(20,184,166,0.2)' },
  { bg: 'rgba(14,165,233,0.08)', border: 'rgba(14,165,233,0.2)' },
];

/* Scatter each bubble around a circle so it can "assemble" inward on reveal */
function burstOffset(index: number, total: number) {
  const angle = (index / total) * Math.PI * 2;
  const radius = 180 + (index % 3) * 60;
  // Rounded so the SSR-rendered inline style string matches the client's
  // exactly — unrounded floats format with different precision on each
  // side and trip a hydration mismatch warning.
  return { x: Math.round(Math.cos(angle) * radius), y: Math.round(Math.sin(angle) * radius) };
}

/* ── Centered bubble ── */
function SkillBubble({ label, index, total }: { label: string; index: number; total: number }) {
  const { x, y } = burstOffset(index, total);
  const color = bubbleColors[index % bubbleColors.length];

  return (
    <motion.div
      initial={{ opacity: 0, x, y, scale: 0.3, rotate: index % 2 === 0 ? -30 : 30 }}
      whileInView={{ opacity: 1, x: 0, y: 0, scale: 1, rotate: 0 }}
      viewport={{ once: true, margin: '-100px' }}
      transition={{ type: 'spring', stiffness: 110, damping: 14, mass: 0.7, delay: index * 0.02 }}
    >
      <span
        className="animate-float inline-block"
        style={{
          animationDuration: `${3 + (index % 5) * 0.4}s`,
          animationDelay: `${(index % 7) * 0.35}s`,
        }}
      >
        <motion.span
          className="skill-pill"
          style={{ background: color.bg, borderColor: color.border }}
          whileHover={{ scale: 1.14, y: -6 }}
        >
          <SkillIcon label={label} className="w-3.5 h-3.5 shrink-0" />
          {label}
        </motion.span>
      </span>
    </motion.div>
  );
}

/* ── Container variants for stagger ── */
const containerVariants: Variants = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.13 } },
};

const cardVariants: Variants = {
  hidden: { opacity: 0, y: 32, scale: 0.97 },
  visible: { opacity: 1, y: 0, scale: 1, transition: { duration: 0.7, ease: [0.25, 1, 0.5, 1] } },
};

export default function Stack() {
  return (
    <section id="stack" className="py-28 md:py-40 relative z-10 overflow-hidden">

      {/* Background orbs */}
      <div
        className="absolute -bottom-1/4 -left-1/4 w-[600px] h-[600px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(circle, var(--orb-1) 0%, transparent 70%)',
          filter: 'blur(100px)',
          opacity: 0.4,
        }}
      />
      <div
        className="absolute top-1/4 right-0 w-[400px] h-[400px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(circle, var(--orb-3) 0%, transparent 70%)',
          filter: 'blur(80px)',
          opacity: 0.35,
        }}
      />

      {/* ── Section header ── */}
      <div className="px-6 md:px-8">
        <div className="max-w-6xl mx-auto mb-20 text-center">
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-100px' }}
            transition={{ duration: 0.9, ease: [0.25, 1, 0.5, 1] }}
          >
            <span className="section-label justify-center">Technologies</span>
            <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-5 tracking-tighter">
              Tech <span className="text-gradient">stack.</span>
            </h2>
            <p className="section-subtitle font-light md:text-xl mx-auto">
              My daily-driven toolkit for crafting performant, scalable modern web applications.
            </p>
          </motion.div>
        </div>
      </div>

      {/* ── Centered skill bubble cloud ── */}
      <div className="px-6 md:px-8 mb-20 relative">
        {/* Glow anchoring the cloud to the center */}
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[520px] h-[320px] rounded-full pointer-events-none -z-10"
          style={{
            background: 'radial-gradient(ellipse, var(--accent-light) 0%, transparent 70%)',
            filter: 'blur(60px)',
          }}
        />
        <div className="flex flex-wrap items-center justify-center gap-3 max-w-4xl mx-auto">
          {allSkills.map((skill, i) => (
            <SkillBubble key={skill} label={skill} index={i} total={allSkills.length} />
          ))}
        </div>
      </div>

      {/* Section sep */}
      <div className="px-6 md:px-8 mb-16">
        <div className="max-w-6xl mx-auto">
          <div className="section-sep" />
        </div>
      </div>

      {/* ── Category cards ── */}
      <div className="px-6 md:px-8">
        <div className="max-w-6xl mx-auto">
          <motion.div
            variants={containerVariants}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true, margin: '-60px' }}
            className="grid grid-cols-1 md:grid-cols-2 gap-6"
          >
            {stackCategories.map((cat) => (
              <motion.div
                key={cat.title}
                variants={cardVariants}
                className="card group p-8 md:p-10 relative overflow-hidden"
              >
                {/* Top accent bar */}
                <div
                  className="absolute top-0 left-0 right-0 h-[3px] rounded-t-[20px] opacity-70 group-hover:opacity-100 transition-opacity duration-400"
                  style={{ background: cat.gradient }}
                />

                {/* Background gradient on hover */}
                <div
                  className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
                  style={{
                    background: `radial-gradient(circle at 20% 20%, ${cat.accentColor}0a 0%, transparent 60%)`,
                  }}
                />

                {/* Number watermark */}
                <span
                  className="absolute top-3 right-6 text-7xl font-black select-none pointer-events-none transition-all duration-500 group-hover:opacity-15"
                  style={{
                    color: cat.accentColor,
                    opacity: 0.07,
                    letterSpacing: '-0.06em',
                    lineHeight: 1,
                    fontFamily: 'Inter, sans-serif',
                  }}
                >
                  {cat.number}
                </span>

                <div className="relative z-10">
                  {/* Emoji + Title row */}
                  <div className="flex items-center gap-3 mb-2">
                    <div
                      className="w-10 h-10 rounded-xl flex items-center justify-center shadow-sm"
                      style={{
                        background: `${cat.accentColor}15`,
                        border: `1px solid ${cat.accentColor}30`,
                      }}
                    >
                      <cat.categoryIcon className="w-5 h-5" style={{ color: cat.accentColor }} />
                    </div>
                    <h3
                      className="text-xl font-display font-bold tracking-tight transition-colors duration-300 group-hover:text-accent"
                      style={{ color: 'var(--text-primary)', letterSpacing: '-0.03em' }}
                    >
                      {cat.title}
                    </h3>
                  </div>
                  <p
                    className="text-[0.875rem] mb-7 font-medium leading-relaxed pl-[52px]"
                    style={{ color: 'var(--text-tertiary)' }}
                  >
                    {cat.description}
                  </p>

                  <div className="flex flex-wrap gap-2">
                    {cat.skills.map((skill, i) => (
                      <motion.span
                        key={skill}
                        className="skill-badge inline-flex items-center gap-1.5"
                        initial={{ opacity: 0, scale: 0.82 }}
                        whileInView={{ opacity: 1, scale: 1 }}
                        viewport={{ once: true }}
                        transition={{ duration: 0.3, delay: i * 0.045 }}
                        whileHover={{ scale: 1.07, y: -1 }}
                        style={{
                          background: `${cat.accentColor}0d`,
                          borderColor: `${cat.accentColor}28`,
                          color: cat.accentColor,
                        }}
                      >
                        <SkillIcon label={skill} className="w-3.5 h-3.5 shrink-0" />
                        {skill}
                      </motion.span>
                    ))}
                  </div>
                </div>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}
