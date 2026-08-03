'use client';

/**
 * Scene3D — React Three Fiber background scene for the Hero section.
 *
 * Accent colours are the hardcoded equivalents of globals.css dark-mode variables:
 *   --accent:   #60a5fa
 *   --accent-2: #a78bfa
 *   --accent-3: #22d3ee
 *
 * Lazy-loaded via next/dynamic({ ssr: false }) in Hero.tsx so Three.js never
 * enters the initial JS bundle or the SSR path.
 *
 * TypeScript notes:
 *  - Import concrete types from 'three' directly (e.g. `Mesh`, `Group`) — the
 *    namespace import `import * as THREE` is deliberately avoided because
 *    @types/three ≥ 0.170 restructured its exports in a way that breaks the
 *    THREE.Mesh shorthand in some moduleResolution modes.
 *  - The JSX intrinsics (<mesh>, <group>, <icosahedronGeometry>, …) are
 *    augmented by @react-three/fiber. The triple-slash reference below pulls in
 *    those augmentations so TypeScript recognises them.
 */

/// <reference types="@react-three/fiber" />

import { useRef, useEffect } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { Float } from '@react-three/drei';
import type { Mesh, Group } from 'three';
import { DoubleSide } from 'three';

// ─────────────────────────────────────────────────────────────
// Shape configuration
// ─────────────────────────────────────────────────────────────

type GeoType = 'icosahedron' | 'torus' | 'octahedron' | 'box' | 'cone';

interface ShapeCfg {
  id: string;
  geo: GeoType;
  args: number[];
  pos: [number, number, number];
  initRot: [number, number, number];
  /** per-frame rotation delta on each axis */
  rotSpeed: [number, number, number];
  floatSpeed: number;
  floatIntensity: number;
  color: string;
  wireframe: boolean;
  opacity: number;
  metalness: number;
  roughness: number;
}

const shapes: ShapeCfg[] = [
  // 1 ── Icosahedron  (wireframe, accent primary, right-centre)
  {
    id: 'ico',
    geo: 'icosahedron',
    args: [0.75, 1],
    pos: [2.0, 0.3, -0.5],
    initRot: [0.3, 0.5, 0],
    rotSpeed: [0.003, 0.005, 0.001],
    floatSpeed: 1.2,
    floatIntensity: 0.5,
    color: '#60a5fa',
    wireframe: true,
    opacity: 0.70,
    metalness: 0,
    roughness: 0.5,
  },
  // 2 ── Torus  (glass, accent-2, upper-left)
  {
    id: 'tor',
    geo: 'torus',
    args: [0.5, 0.16, 12, 36],
    pos: [-2.0, 0.9, -0.8],
    initRot: [0.8, 0.2, 0.1],
    rotSpeed: [0.002, 0.004, 0.003],
    floatSpeed: 0.8,
    floatIntensity: 0.4,
    color: '#a78bfa',
    wireframe: false,
    opacity: 0.22,
    metalness: 0.3,
    roughness: 0.1,
  },
  // 3 ── Octahedron  (glass, accent-3, lower-right)
  {
    id: 'oct',
    geo: 'octahedron',
    args: [0.6, 0],
    pos: [2.6, -1.1, 0.4],
    initRot: [0.2, 0.8, 0.3],
    rotSpeed: [0.004, 0.002, 0.005],
    floatSpeed: 1.5,
    floatIntensity: 0.45,
    color: '#22d3ee',
    wireframe: false,
    opacity: 0.25,
    metalness: 0.5,
    roughness: 0.15,
  },
  // 4 ── Thin box  ("UI panel", dark glass, left-lower)
  {
    id: 'box',
    geo: 'box',
    args: [1.2, 0.72, 0.07],
    pos: [-2.4, -0.7, -0.3],
    initRot: [0.1, -0.3, 0.15],
    rotSpeed: [0.001, 0.003, 0.002],
    floatSpeed: 0.6,
    floatIntensity: 0.35,
    color: '#60a5fa',
    wireframe: false,
    opacity: 0.15,
    metalness: 0.85,
    roughness: 0.05,
  },
  // 5 ── Low-poly cone  (wireframe, accent-2, bottom-centre)
  {
    id: 'con',
    geo: 'cone',
    args: [0.4, 1.0, 5],
    pos: [0.4, -1.8, 0.6],
    initRot: [0, 0, 0.3],
    rotSpeed: [0.003, 0.006, 0.001],
    floatSpeed: 1.0,
    floatIntensity: 0.5,
    color: '#a78bfa',
    wireframe: true,
    opacity: 0.55,
    metalness: 0,
    roughness: 0.5,
  },
];

// ─────────────────────────────────────────────────────────────
// Individual shape mesh — handles its own continuous rotation
// ─────────────────────────────────────────────────────────────

function Shape3D({ cfg }: { cfg: ShapeCfg }) {
  const meshRef = useRef<Mesh>(null!);

  useFrame(() => {
    meshRef.current.rotation.x += cfg.rotSpeed[0];
    meshRef.current.rotation.y += cfg.rotSpeed[1];
    meshRef.current.rotation.z += cfg.rotSpeed[2];
  });

  return (
    <Float
      speed={cfg.floatSpeed}
      rotationIntensity={0}          // rotation handled by useFrame above
      floatIntensity={cfg.floatIntensity}
      floatingRange={[-0.12, 0.12]}
    >
      <mesh ref={meshRef} position={cfg.pos} rotation={cfg.initRot}>
        {/* Geometry — switched by string type */}
        {cfg.geo === 'icosahedron' && (
          <icosahedronGeometry args={cfg.args as [number, number]} />
        )}
        {cfg.geo === 'torus' && (
          <torusGeometry args={cfg.args as [number, number, number, number]} />
        )}
        {cfg.geo === 'octahedron' && (
          <octahedronGeometry args={cfg.args as [number, number]} />
        )}
        {cfg.geo === 'box' && (
          <boxGeometry args={cfg.args as [number, number, number]} />
        )}
        {cfg.geo === 'cone' && (
          <coneGeometry args={cfg.args as [number, number, number]} />
        )}

        <meshStandardMaterial
          color={cfg.color}
          wireframe={cfg.wireframe}
          transparent
          opacity={cfg.opacity}
          metalness={cfg.metalness}
          roughness={cfg.roughness}
          side={DoubleSide}
        />
      </mesh>
    </Float>
  );
}

// ─────────────────────────────────────────────────────────────
// Scene root — damps mouse movement onto the whole group
// Mouse is tracked via window events so the canvas can be
// pointer-events-none and never block hero button clicks.
// ─────────────────────────────────────────────────────────────

function SceneContents() {
  const groupRef = useRef<Group>(null!);
  // Raw NDC pointer (-1 … +1)
  const raw    = useRef({ x: 0, y: 0 });
  // Smoothly lerped copy
  const lerped = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      raw.current.x =  (e.clientX / window.innerWidth)  * 2 - 1;
      raw.current.y = -(e.clientY / window.innerHeight) * 2 + 1;
    };
    window.addEventListener('mousemove', onMove, { passive: true });
    return () => window.removeEventListener('mousemove', onMove);
  }, []);

  useFrame(() => {
    // lf = 0.04 → silky smooth parallax, not twitchy
    const lf = 0.04;
    lerped.current.x += (raw.current.x * 0.22 - lerped.current.x) * lf;
    lerped.current.y += (raw.current.y * 0.22 - lerped.current.y) * lf;

    groupRef.current.rotation.y =  lerped.current.x;
    groupRef.current.rotation.x = -lerped.current.y;
  });

  return (
    <group ref={groupRef}>
      {shapes.map((s) => (
        <Shape3D key={s.id} cfg={s} />
      ))}
    </group>
  );
}

// ─────────────────────────────────────────────────────────────
// Canvas export — imported lazily by Hero.tsx via next/dynamic
// ─────────────────────────────────────────────────────────────

export default function Scene3D() {
  return (
    <Canvas
      dpr={[1, 1.5]}
      camera={{ position: [0, 0, 7], fov: 50 }}
      gl={{
        alpha: true,
        antialias: true,
        powerPreference: 'high-performance',
      }}
      style={{ background: 'transparent', width: '100%', height: '100%' }}
    >
      {/* Soft ambient so wireframes are always visible */}
      <ambientLight intensity={0.35} />
      {/* Main directional for depth/specular on the glass shapes */}
      <directionalLight position={[5, 6, 4]} intensity={1.1} />
      {/* Accent-coloured point light for a purple rim glow */}
      <pointLight position={[-4, -3, 3]} intensity={0.6} color="#60a5fa" />

      <SceneContents />
    </Canvas>
  );
}
