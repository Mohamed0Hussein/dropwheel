'use client';

import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import { Send, CheckCircle, AlertCircle, Mail, MapPin, Phone } from 'lucide-react';
import { submitContactForm, ContactFormData } from '@/lib/api';

export default function Contact() {
  const [formData, setFormData] = useState<ContactFormData>({
    name: '',
    email: '',
    message: ''
  });

  const mutation = useMutation({
    mutationFn: submitContactForm,
    onSuccess: () => {
      setFormData({ name: '', email: '', message: '' });
    },
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    mutation.mutate(formData);
  };

  return (
    <section id="contact" className="py-20 relative overflow-hidden">
      <div className="absolute -bottom-20 -right-20 w-96 h-96 bg-pink-500/10 rounded-full blur-[100px]" />

      <div className="container mx-auto px-6 relative z-10">
        <motion.div
           initial={{ opacity: 0, y: 20 }}
           whileInView={{ opacity: 1, y: 0 }}
           viewport={{ once: true }}
           transition={{ duration: 0.6 }}
           className="text-center mb-16"
        >
          <h2 className="text-4xl font-bold mb-4">Get In <span className="text-gradient">Touch</span></h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Have a project in mind or want to collaborate? Feel free to reach out.
          </p>
        </motion.div>

        <div className="flex flex-col lg:flex-row gap-12 max-w-6xl mx-auto">
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="lg:w-1/3 space-y-8"
          >
            <div className="glass p-8 rounded-2xl border border-white/5 space-y-6">
              <h3 className="text-2xl font-bold">Contact Info</h3>
              
              <div className="flex items-start gap-4 text-gray-300">
                <div className="p-3 bg-white/5 rounded-lg">
                  <Mail className="w-6 h-6 text-indigo-400" />
                </div>
                <div>
                  <h4 className="font-semibold text-white">Email</h4>
                  <p>mohamed.hussein0x1@gmail.com</p>
                </div>
              </div>

              <div className="flex items-start gap-4 text-gray-300">
                <div className="p-3 bg-white/5 rounded-lg">
                  <MapPin className="w-6 h-6 text-purple-400" />
                </div>
                <div>
                  <h4 className="font-semibold text-white">Location</h4>
                  <p>Cairo, Egypt</p>
                </div>
              </div>

              <div className="flex items-start gap-4 text-gray-300">
                <div className="p-3 bg-white/5 rounded-lg">
                  <Phone className="w-6 h-6 text-pink-400" />
                </div>
                <div>
                  <h4 className="font-semibold text-white">Phone</h4>
                  <p>+201149247110</p>
                </div>
              </div>
            </div>
            
            <div className="p-8 rounded-2xl bg-linear-to-br from-indigo-900/50 to-purple-900/50 border border-white/10">
              <h3 className="text-xl font-bold mb-2">Open to Opportunities</h3>
              <p className="text-gray-400 text-sm">
                I am currently available for freelance work and full-time positions. Let&apos;s build something amazing together.
              </p>
            </div>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="lg:w-2/3 glass p-8 md:p-10 rounded-2xl border border-white/5"
          >
            <form onSubmit={handleSubmit} className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <label htmlFor="name" className="block text-sm font-medium text-gray-400 mb-2">Name</label>
                  <input
                    id="name"
                    type="text"
                    required
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full bg-black/20 border border-white/10 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-purple-500 transition-colors"
                    placeholder="Your Name"
                  />
                </div>
                <div>
                  <label htmlFor="email" className="block text-sm font-medium text-gray-400 mb-2">Email</label>
                  <input
                    id="email"
                    type="email"
                    required
                    value={formData.email}
                    onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                    className="w-full bg-black/20 border border-white/10 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-purple-500 transition-colors"
                    placeholder="your@email.com"
                  />
                </div>
              </div>

              <div>
                <label htmlFor="message" className="block text-sm font-medium text-gray-400 mb-2">Message</label>
                <textarea
                  id="message"
                  required
                  rows={6}
                  value={formData.message}
                  onChange={(e) => setFormData({ ...formData, message: e.target.value })}
                  className="w-full bg-black/20 border border-white/10 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-purple-500 transition-colors resize-none"
                  placeholder="Tell me about your project..."
                />
              </div>

              <button
                type="submit"
                disabled={mutation.isPending || mutation.isSuccess}
                className="w-full py-4 rounded-lg bg-linear-to-r from-indigo-600 to-purple-600 text-white font-bold hover:opacity-90 transition-opacity flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {mutation.isPending ? (
                  <span className="animate-spin text-xl">◌</span>
                ) : mutation.isSuccess ? (
                  <>Sent Successfully <CheckCircle size={20} /></>
                ) : mutation.isError ? (
                  <>Failed to Send <AlertCircle size={20} /></>
                ) : (
                  <>Send Message <Send size={20} /></>
                )}
              </button>
              
              {mutation.isError && (
                <p className="text-red-400 text-sm text-center">
                  Something went wrong. Please try again.
                </p>
              )}
            </form>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
