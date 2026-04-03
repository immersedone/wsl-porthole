<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import QRCode from "qrcode";

const props = defineProps<{ url: string; size?: number }>();
const canvas = ref<HTMLCanvasElement | null>(null);
const dim = props.size ?? 200;

async function render() {
  if (!canvas.value) return;
  try {
    await QRCode.toCanvas(canvas.value, props.url, {
      width: dim,
      margin: 2,
      color: { dark: "#000000", light: "#ffffff" },
    });
  } catch (e) {
    console.error("QR generation failed:", e);
  }
}

onMounted(render);
watch(() => props.url, render);
</script>

<template>
  <div class="inline-block rounded-lg overflow-hidden bg-white p-1">
    <canvas ref="canvas" :width="dim" :height="dim" />
  </div>
</template>
