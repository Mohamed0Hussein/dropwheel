import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import QueryProvider from "@/components/QueryProvider";

const inter = Inter({
  variable: "--font-geist-sans",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "Mohamed Hussein — Frontend Engineer",
  description:
    "Frontend Engineer specializing in React, Next.js & TypeScript. Building high-performance web applications with pixel-perfect interfaces. Based in Cairo, Egypt.",
  keywords: [
    "Frontend Engineer",
    "React Developer",
    "Next.js",
    "TypeScript",
    "Web Developer",
    "Mohamed Hussein",
    "Cairo",
    "Egypt",
  ],
  authors: [{ name: "Mohamed Hussein" }],
  openGraph: {
    title: "Mohamed Hussein — Frontend Engineer",
    description:
      "Frontend Engineer specializing in React, Next.js & TypeScript. Building high-performance web applications with pixel-perfect interfaces.",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "Mohamed Hussein — Frontend Engineer",
    description:
      "Frontend Engineer specializing in React, Next.js & TypeScript. Building high-performance web applications.",
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                try {
                  var theme = localStorage.getItem('theme');
                  if (!theme) {
                    theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
                  }
                  document.documentElement.setAttribute('data-theme', theme);
                } catch (e) {
                  document.documentElement.setAttribute('data-theme', 'light');
                }
              })();
            `,
          }}
        />
      </head>
      <body className={`${inter.variable} antialiased`}>
        <div className="mesh-bg" />
        <div className="mesh-bg-third" />
        <div className="grid-overlay" />
        <div className="noise" />
        <QueryProvider>{children}</QueryProvider>
      </body>
    </html>
  );
}
