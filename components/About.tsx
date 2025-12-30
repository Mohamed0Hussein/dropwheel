'use client';

import { useQuery } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import { Download, Calendar, GraduationCap, Briefcase, Loader2 } from 'lucide-react';
import Image from 'next/image';
import { fetchGitHubStats } from '@/lib/api';

export default function About() {
  const { data: stats, isLoading } = useQuery({
    queryKey: ['github-stats'],
    queryFn: fetchGitHubStats,
    staleTime: 1000 * 60 * 60,
  });

  const yearsOnGitHub = stats?.createdAt 
    ? new Date().getFullYear() - new Date(stats.createdAt).getFullYear()
    : 7;

  return (
    <section id="about" className="py-20 relative overflow-hidden">
      <div className="absolute top-0 right-0 w-96 h-96 bg-indigo-600/10 rounded-full blur-[120px] -z-10" />

      <div className="container mx-auto px-6">
        <div className="flex flex-col lg:flex-row gap-16 items-center">
          <motion.div
            initial={{ opacity: 0, x: -50 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="lg:w-1/3 flex justify-center"
          >
            <div className="relative group">
              <div className="absolute -inset-4 bg-linear-to-r from-indigo-500 via-purple-500 to-pink-500 rounded-full opacity-40 blur-xl group-hover:opacity-60 transition-opacity duration-500" />
              <div className="relative w-64 h-64 md:w-80 md:h-80 rounded-full overflow-hidden border-4 border-white/10 shadow-2xl">
                <Image 
                  src={stats?.avatarUrl || "https://avatars.githubusercontent.com/u/28152228?v=4"} 
                  alt="Mohamed Hussein" 
                  fill 
                  className="object-cover"
                  sizes="(max-width: 768px) 256px, 320px"
                />
              </div>
            </div>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: 50 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="lg:w-2/3"
          >
            <h2 className="text-4xl font-bold mb-6">About <span className="text-gradient">Me</span></h2>
            
            <p className="text-gray-300 text-lg leading-relaxed mb-6">
              Hi, I&apos;m <span className="text-white font-semibold">{stats?.name || 'Mohamed Hussein'}</span>, a passionate Full Stack Developer based in {stats?.location || 'Cairo, Egypt'}. I&apos;m a graduate of the Faculty of Engineering at Ain Shams University (2020), and I specialize in building modern, performant, and user-friendly web applications.
            </p>
            
            <p className="text-gray-400 leading-relaxed mb-8">
              With expertise in both frontend and backend technologies, I love turning complex problems into elegant solutions. I&apos;m proficient in React, Next.js, Node.js, Express, MongoDB, and Tailwind CSS. I&apos;m always eager to learn new technologies and take on challenging projects that push my skills to the next level.
            </p>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
              <div className="glass p-4 rounded-xl text-center border border-white/5 hover:border-white/10 transition-colors">
                {isLoading ? (
                  <Loader2 className="w-6 h-6 animate-spin text-indigo-400 mx-auto mb-1" />
                ) : (
                  <div className="text-3xl font-bold text-gradient">{stats?.publicRepos || 37}+</div>
                )}
                <div className="text-sm text-gray-400">Public Repos</div>
              </div>
              <div className="glass p-4 rounded-xl text-center border border-white/5 hover:border-white/10 transition-colors">
                <div className="text-3xl font-bold text-gradient">{yearsOnGitHub}+</div>
                <div className="text-sm text-gray-400">Years Exp</div>
              </div>
              <div className="glass p-4 rounded-xl text-center border border-white/5 hover:border-white/10 transition-colors">
                {isLoading ? (
                  <Loader2 className="w-6 h-6 animate-spin text-indigo-400 mx-auto mb-1" />
                ) : (
                  <div className="text-3xl font-bold text-gradient">{stats?.followers || 14}</div>
                )}
                <div className="text-sm text-gray-400">Followers</div>
              </div>
              <div className="glass p-4 rounded-xl text-center border border-white/5 hover:border-white/10 transition-colors">
                <div className="text-3xl font-bold text-gradient">∞</div>
                <div className="text-sm text-gray-400">Passion</div>
              </div>
            </div>

            <div className="flex flex-wrap gap-4 mb-8">
              <div className="flex items-center gap-2 text-gray-300 bg-white/5 px-4 py-2 rounded-full text-sm">
                <GraduationCap size={18} className="text-indigo-400" />
                <span>Ain Shams University 2020</span>
              </div>
              <div className="flex items-center gap-2 text-gray-300 bg-white/5 px-4 py-2 rounded-full text-sm">
                <Briefcase size={18} className="text-green-400" />
                <span className="text-green-400">Open to Opportunities</span>
              </div>
              <div className="flex items-center gap-2 text-gray-300 bg-white/5 px-4 py-2 rounded-full text-sm">
                <Calendar size={18} className="text-purple-400" />
                <span>Since 2017</span>
              </div>
            </div>

            <a
              href="#contact"
              className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-linear-to-r from-indigo-600 to-purple-600 text-white font-semibold hover:opacity-90 transition-opacity shadow-lg shadow-indigo-500/30"
            >
              <Download size={20} />
              Let&apos;s Work Together
            </a>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
