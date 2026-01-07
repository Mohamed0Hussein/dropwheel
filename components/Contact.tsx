'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { ArrowUpRight } from 'lucide-react';

export default function Contact() {
  return (
    <section id="contact" className="py-32 px-6 md:px-16 lg:px-24 min-h-[60vh] flex items-center">
      <div className="max-w-6xl mx-auto w-full">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center"
        >
          <span className="text-gray-500 text-sm uppercase tracking-widest mb-6 block">Contact</span>
          
          <h2 className="font-display text-4xl sm:text-5xl md:text-6xl lg:text-7xl text-white mb-8 leading-tight">
            Have a project in mind?
          </h2>
          
          <p className="text-gray-500 text-lg max-w-xl mx-auto mb-12">
            I&apos;m always open to discussing new projects, creative ideas, or opportunities to be part of your vision.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-6">
            <Link
              href="mailto:mohamed.hussein0x1@gmail.com"
              className="group inline-flex items-center gap-3 text-2xl md:text-3xl text-white hover:text-gray-400 transition-colors"
            >
              <span className="border-b-2 border-white group-hover:border-gray-400 transition-colors pb-1">
                mohamed.hussein0x1@gmail.com
              </span>
              <ArrowUpRight className="w-6 h-6 group-hover:rotate-45 transition-transform" />
            </Link>
          </div>

          {/* Social Links */}
          <motion.div
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.3 }}
            className="mt-16 flex items-center justify-center gap-8"
          >
            <Link
              href="https://github.com/Mohamed0Hussein"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-500 hover:text-white transition-colors text-sm uppercase tracking-wider"
            >
              GitHub
            </Link>
            <span className="text-gray-700">•</span>
            <Link
              href="https://www.linkedin.com/in/mohamed-hussein-0x1/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-500 hover:text-white transition-colors text-sm uppercase tracking-wider"
            >
              LinkedIn
            </Link>
            <span className="text-gray-700">•</span>
            <Link
              href="tel:+201149247110"
              className="text-gray-500 hover:text-white transition-colors text-sm uppercase tracking-wider"
            >
              Phone
            </Link>
          </motion.div>

          {/* Footer */}
          <motion.div
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.5 }}
            className="mt-24 pt-8 border-t border-white/5"
          >
            <p className="text-gray-600 text-sm">
              © {new Date().getFullYear()} Mohamed Hussein. All rights reserved.
            </p>
          </motion.div>
        </motion.div>
      </div>
    </section>
  );
}
