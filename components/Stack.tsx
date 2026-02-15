'use client';

import { motion } from 'framer-motion';

const stackCategories = [
  {
    title: 'Frontend',
    description: 'Building pixel-perfect, performant interfaces',
    skills: [
      'JavaScript',
      'TypeScript',
      'React',
      'Next.js',
      'Redux',
      'Tailwind CSS',
      'Material UI',
      'Framer Motion',
      'HTML5',
      'CSS3',
    ],
  },
  {
    title: 'Backend',
    description: 'Server-side logic and API design',
    skills: ['Node.js', 'Express.js', 'REST APIs', 'Next.js API Routes'],
  },
  {
    title: 'Database',
    description: 'Data modeling and persistence',
    skills: ['MongoDB', 'Mongoose', 'PostgreSQL'],
  },
  {
    title: 'DevOps & Tools',
    description: 'Workflow optimization and deployment',
    skills: ['Git', 'GitHub', 'Docker', 'Vercel', 'Vite', 'VS Code', 'Postman', 'Figma'],
  },
];

export default function Stack() {
  return (
    <section id="stack" className="py-24 md:py-32 px-6 md:px-8">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mb-12"
        >
          <span className="section-label">Technologies</span>
          <h2 className="section-title text-4xl md:text-5xl lg:text-6xl mb-4">
            Tech stack
          </h2>
          <p className="section-subtitle">
            My daily toolkit for building modern web applications.
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {stackCategories.map((category, catIndex) => (
            <motion.div
              key={category.title}
              initial={{ opacity: 0, y: 24 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.5, delay: catIndex * 0.1 }}
              className="card p-6 md:p-7"
            >
              <div className="mb-5">
                <h3
                  className="text-lg font-bold mb-1"
                  style={{ color: 'var(--text-primary)', letterSpacing: '-0.01em' }}
                >
                  {category.title}
                </h3>
                <p
                  className="text-sm"
                  style={{ color: 'var(--text-tertiary)' }}
                >
                  {category.description}
                </p>
              </div>
              <div className="flex flex-wrap gap-2">
                {category.skills.map((skill) => (
                  <span key={skill} className="skill-badge">
                    {skill}
                  </span>
                ))}
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
