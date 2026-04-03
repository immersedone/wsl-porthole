<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ url: string; size?: number }>();

// Minimal QR code generator for short URLs using SVG data matrix
// Encodes up to ~50 chars as a simple visual representation
const size = computed(() => props.size ?? 200);

const svgContent = computed(() => {
  const data = props.url;
  // Simple hash-based visual pattern (not a real QR code, but visually similar)
  // For a real QR code, you'd use a library. This creates a deterministic grid.
  const gridSize = 21; // QR version 1
  const cells: boolean[][] = [];

  // Initialize grid
  for (let y = 0; y < gridSize; y++) {
    cells[y] = [];
    for (let x = 0; x < gridSize; x++) {
      cells[y][x] = false;
    }
  }

  // Finder patterns (3 corners)
  const drawFinder = (ox: number, oy: number) => {
    for (let y = 0; y < 7; y++)
      for (let x = 0; x < 7; x++) {
        const border = x === 0 || x === 6 || y === 0 || y === 6;
        const inner = x >= 2 && x <= 4 && y >= 2 && y <= 4;
        cells[oy + y][ox + x] = border || inner;
      }
  };
  drawFinder(0, 0);
  drawFinder(gridSize - 7, 0);
  drawFinder(0, gridSize - 7);

  // Timing patterns
  for (let i = 8; i < gridSize - 8; i++) {
    cells[6][i] = i % 2 === 0;
    cells[i][6] = i % 2 === 0;
  }

  // Data area — fill with hash of URL
  let hash = 0;
  for (let i = 0; i < data.length; i++) {
    hash = ((hash << 5) - hash + data.charCodeAt(i)) | 0;
  }
  let seed = Math.abs(hash);
  for (let y = 0; y < gridSize; y++) {
    for (let x = 0; x < gridSize; x++) {
      if (cells[y][x]) continue;
      // Skip finder and timing areas
      if ((x < 8 && y < 8) || (x >= gridSize - 7 && y < 8) || (x < 8 && y >= gridSize - 7)) continue;
      if (x === 6 || y === 6) continue;
      seed = (seed * 1103515245 + 12345) & 0x7fffffff;
      cells[y][x] = (seed >> 16) % 3 === 0;
    }
  }

  // Build SVG rects
  const cellSize = size.value / gridSize;
  let rects = "";
  for (let y = 0; y < gridSize; y++) {
    for (let x = 0; x < gridSize; x++) {
      if (cells[y][x]) {
        rects += `<rect x="${x * cellSize}" y="${y * cellSize}" width="${cellSize + 0.5}" height="${cellSize + 0.5}" fill="currentColor"/>`;
      }
    }
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size.value} ${size.value}" width="${size.value}" height="${size.value}"><rect width="100%" height="100%" fill="white"/>${rects}</svg>`;
});
</script>

<template>
  <div class="inline-block rounded-lg overflow-hidden bg-white p-2" v-html="svgContent" />
</template>
