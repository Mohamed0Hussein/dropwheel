'use client';

import { motion } from 'framer-motion';

export default function About() {
  return (
    <section id="about-me" className="py-32 px-6 md:px-16 lg:px-24">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="grid grid-cols-1 lg:grid-cols-2 gap-16"
        >
          {/* Section Title */}
          <div>
            <span className="text-gray-500 text-sm uppercase tracking-widest mb-4 block">About</span>
            <h2 className="font-display text-5xl md:text-6xl lg:text-7xl text-white leading-tight">
              This is me
            </h2>
          </div>

          {/* Content */}
          <div className="flex flex-col justify-center">
            <p className="text-xl md:text-2xl text-gray-300 leading-relaxed mb-8">
              I&apos;m a <span className="text-white">Senior Frontend Engineer</span> with over 4 years of experience building high-quality web applications.
            </p>
            <p className="text-gray-500 leading-relaxed mb-8">
              I specialize in <span className="text-gray-300">React</span>, <span className="text-gray-300">Next.js</span>, <span className="text-gray-300">TypeScript</span>, and <span className="text-gray-300">Tailwind CSS</span>. I focus on creating scalable architecture, pixel-perfect interfaces, and seamless user experiences. My expertise extends to performance optimization, internationalization, and building reusable component libraries.
            </p>
            <p className="text-gray-500 leading-relaxed">
              When I&apos;m not coding, you&apos;ll find me exploring new technologies, contributing to open-source projects, or sharing knowledge with the developer community.
            </p>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
