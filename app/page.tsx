import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import About from "@/components/About";
import Stack from "@/components/Stack";
import Experience from "@/components/Experience";
import Projects from "@/components/Projects";
import Contact from "@/components/Contact";
import SideElements from "@/components/SideElements";
import Footer from "@/components/Footer";
import ScrollTilt from "@/components/ScrollTilt";

export default function Home() {
  return (
    <main id="main-content" className="min-h-screen relative">
      <Navbar />
      <SideElements />
      {/* Hero is in view on load, so it only folds away on the way out. */}
      <ScrollTilt tiltIn={false}>
        <Hero />
      </ScrollTilt>
      <ScrollTilt>
        <About />
      </ScrollTilt>
      <ScrollTilt>
        <Projects />
      </ScrollTilt>
      <ScrollTilt>
        <Stack />
      </ScrollTilt>
      <ScrollTilt>
        <Experience />
      </ScrollTilt>
      {/* Contact sits above the footer at the end of the page, so it never
          scrolls far enough to leave; folding it out would leave it tilted. */}
      <ScrollTilt tiltOut={false}>
        <Contact />
      </ScrollTilt>
      <Footer />
    </main>
  );
}
