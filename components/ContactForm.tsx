'use client';

import { AnimatePresence, motion } from 'framer-motion';
import { CheckCircle2, Loader2, Send } from 'lucide-react';
import { FormEvent, useState } from 'react';

type FieldErrors = Partial<Record<'name' | 'email' | 'message', string>>;
type Status = 'idle' | 'submitting' | 'success' | 'error';

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function validate(values: { name: string; email: string; message: string }): FieldErrors {
  const errors: FieldErrors = {};
  if (!values.name.trim()) errors.name = 'Please enter your name.';
  if (!values.email.trim()) errors.email = 'Please enter your email.';
  else if (!EMAIL_RE.test(values.email)) errors.email = 'Please enter a valid email address.';
  if (!values.message.trim()) errors.message = 'Please enter a message.';
  else if (values.message.trim().length < 10) errors.message = 'Message should be at least 10 characters.';
  return errors;
}

export default function ContactForm() {
  const [values, setValues] = useState({ name: '', email: '', message: '' });
  const [errors, setErrors] = useState<FieldErrors>({});
  const [status, setStatus] = useState<Status>('idle');
  const [serverError, setServerError] = useState<string | null>(null);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const validationErrors = validate(values);
    setErrors(validationErrors);
    if (Object.keys(validationErrors).length > 0) return;

    setStatus('submitting');
    setServerError(null);

    try {
      const res = await fetch('/api/contact', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(values),
      });
      const data = await res.json();
      if (!res.ok || !data.success) {
        throw new Error(data.error || 'Something went wrong. Please try again.');
      }
      setStatus('success');
      setValues({ name: '', email: '', message: '' });
    } catch (err) {
      setStatus('error');
      setServerError(err instanceof Error ? err.message : 'Something went wrong. Please try again.');
    }
  }

  function field(name: keyof typeof values, label: string, type: 'text' | 'email' | 'textarea') {
    const hasError = Boolean(errors[name]);
    const commonProps = {
      id: `contact-${name}`,
      name,
      value: values[name],
      onChange: (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
        setValues((v) => ({ ...v, [name]: e.target.value })),
      'aria-invalid': hasError,
      'aria-describedby': hasError ? `contact-${name}-error` : undefined,
      className: 'w-full px-4 py-3 rounded-xl text-sm outline-none transition-colors duration-300',
      style: {
        background: 'var(--surface-elevated)',
        border: `1.5px solid ${hasError ? '#ef4444' : 'var(--border-strong)'}`,
        color: 'var(--text-primary)',
      } as React.CSSProperties,
    };

    return (
      <div>
        <label htmlFor={commonProps.id} className="block text-xs font-bold uppercase tracking-widest mb-2" style={{ color: 'var(--text-tertiary)' }}>
          {label}
        </label>
        {type === 'textarea' ? (
          <textarea {...commonProps} rows={5} placeholder="Tell me about your project..." />
        ) : (
          <input {...commonProps} type={type} placeholder={label} />
        )}
        {hasError && (
          <p id={`${commonProps.id}-error`} role="alert" className="mt-1.5 text-xs font-medium" style={{ color: '#ef4444' }}>
            {errors[name]}
          </p>
        )}
      </div>
    );
  }

  return (
    <div
      className="relative rounded-[24px] p-7 md:p-8 overflow-hidden"
      style={{
        background: 'var(--glass-bg)',
        border: '1px solid var(--glass-border)',
        backdropFilter: 'blur(24px) saturate(200%)',
        WebkitBackdropFilter: 'blur(24px) saturate(200%)',
        boxShadow: '0 4px 30px rgba(0,0,0,0.15)',
      }}
    >
      <div
        className="absolute -top-24 -right-24 w-64 h-64 rounded-full pointer-events-none"
        style={{ background: 'radial-gradient(circle, var(--orb-1) 0%, transparent 70%)', filter: 'blur(60px)' }}
        aria-hidden="true"
      />

      <AnimatePresence mode="wait">
        {status === 'success' ? (
          <motion.div
            key="success"
            initial={{ opacity: 0, scale: 0.92 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.92 }}
            transition={{ duration: 0.4, ease: [0.25, 1, 0.5, 1] }}
            className="relative z-10 flex flex-col items-center justify-center text-center py-10"
          >
            <motion.div
              initial={{ scale: 0, rotate: -45 }}
              animate={{ scale: 1, rotate: 0 }}
              transition={{ delay: 0.15, type: 'spring', stiffness: 300, damping: 18 }}
              className="mb-5 flex items-center justify-center w-16 h-16 rounded-full"
              style={{ background: 'rgba(34,197,94,0.12)', border: '1px solid rgba(34,197,94,0.3)' }}
            >
              <CheckCircle2 className="w-8 h-8" style={{ color: '#22c55e' }} />
            </motion.div>
            <h3 className="text-xl font-display font-bold mb-2" style={{ color: 'var(--text-primary)' }}>
              Message sent
            </h3>
            <p className="text-sm max-w-xs" style={{ color: 'var(--text-secondary)' }}>
              Thanks for reaching out — I read every message and typically reply within 24 hours.
            </p>
            <button
              type="button"
              onClick={() => setStatus('idle')}
              className="mt-6 text-sm font-semibold underline underline-offset-4"
              style={{ color: 'var(--accent)' }}
            >
              Send another message
            </button>
          </motion.div>
        ) : (
          <motion.form
            key="form"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.3 }}
            onSubmit={handleSubmit}
            className="relative z-10 space-y-5"
            noValidate
          >
            {field('name', 'Name', 'text')}
            {field('email', 'Email', 'email')}
            {field('message', 'Message', 'textarea')}

            {status === 'error' && serverError && (
              <p role="alert" className="text-xs font-medium" style={{ color: '#ef4444' }}>
                {serverError}
              </p>
            )}

            <motion.button
              type="submit"
              disabled={status === 'submitting'}
              whileHover={{ scale: status === 'submitting' ? 1 : 1.02 }}
              whileTap={{ scale: status === 'submitting' ? 1 : 0.97 }}
              className="btn-primary w-full justify-center disabled:opacity-70 disabled:cursor-not-allowed"
            >
              {status === 'submitting' ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Sending...
                </>
              ) : (
                <>
                  <Send className="w-4 h-4" />
                  Send Message
                </>
              )}
            </motion.button>
          </motion.form>
        )}
      </AnimatePresence>
    </div>
  );
}
