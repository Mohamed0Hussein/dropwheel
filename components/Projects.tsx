'use client';

import { motion } from 'framer-motion';
import { ArrowUpRight } from 'lucide-react';
import Link from 'next/link';

const projects = [
  {
    number: '01',
    title: 'Talabat ElBeit',
    category: 'Full-Stack App',
    description: 'A comprehensive food delivery application with menu browsing, cart management, and order tracking.',
    link: 'https://github.com/Mohamed0Hussein/Talabat_ElBeit'
  },
  {
    number: '02',
    title: 'Stories App',
    category: 'Social Media',
    description: 'A social media inspired stories application with ephemeral story content similar to Instagram.',
    link: 'https://stories-app-two.vercel.app'
  },
  {
    number: '03',
    title: 'Login Form',
    category: 'Authentication',
    description: 'Modern, secure authentication form with beautiful UI/UX, validation, and smooth animations.',
    link: 'https://login-form-five-blue.vercel.app'
  },
  {
    number: '04',
    title: 'Space Tourism',
    category: 'Interactive Website',
    description: 'An immersive space tourism website with beautiful animations exploring destinations and technology.',
    link: 'https://github.com/Mohamed0Hussein/Space-tourism'
  },
  {
    number: '05',
    title: 'Age Calculator',
    category: 'Web App',
    description: 'A sleek age calculator that computes exact years, months, and days between dates.',
    link: 'https://age-calculator-sand-rho.vercel.app'
  },
  {
    number: '06',
    title: 'Second Mind',
    category: 'Productivity Tool',
    description: 'Intelligent note-taking application with force-directed graph visualization for connecting ideas.',
    link: 'https://second-brain-ebon.vercel.app/'
  }
];

export default function Projects() {
  return (
    <section id="projects" className="py-32 px-6 md:px-16 lg:px-24 bg-linear-to-b from-transparent via-black/30 to-transparent">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mb-16"
        >
          <span className="text-gray-500 text-sm uppercase tracking-widest mb-4 block">Portfolio</span>
          <h2 className="font-display text-5xl md:text-6xl lg:text-7xl text-white">
            Selected Projects
          </h2>
        </motion.div>

        <div className="project-list space-y-1">
          {projects.map((project, index) => (
            <motion.div
              key={project.number}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: index * 0.1 }}
            >
              <Link
                href={project.link}
                target="_blank"
                rel="noopener noreferrer"
                className="project-item group flex items-center justify-between py-8 border-b border-white/10 hover:border-white/30 transition-all cursor-pointer"
              >
                <div className="flex items-center gap-8">
                  <span className="text-gray-600 text-sm font-mono">{project.number}</span>
                  <div>
                    <h3 className="text-2xl md:text-3xl font-semibold text-white group-hover:text-gray-300 transition-colors">
                      {project.title}
                    </h3>
                    <p className="text-gray-500 text-sm mt-1 hidden md:block">{project.description}</p>
                  </div>
                </div>
                <div className="flex items-center gap-6">
                  <span className="text-gray-500 text-sm hidden lg:block">{project.category}</span>
                  <ArrowUpRight
                    className="w-6 h-6 text-gray-500 group-hover:text-white group-hover:rotate-45 transition-all"
                  />
                </div>
              </Link>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5 }}
          className="mt-16 text-center"
        >
          <Link
            href="https://github.com/Mohamed0Hussein?tab=repositories"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-3 px-8 py-4 border border-white/20 rounded-full text-white hover:bg-white hover:text-black transition-all"
          >
            View All Projects
            <ArrowUpRight className="w-5 h-5" />
          </Link>
        </motion.div>
      </div>
    </section>
  );
}
