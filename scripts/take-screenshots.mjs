import puppeteer from "puppeteer";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(__dirname, "..", "docs", "screenshots");
const baseUrl = "http://localhost:4173";

// Pages to screenshot — map of sidebar button text to filename
const pages = [
  { click: "Port Rules", file: "01-rules.png", wait: 300 },
  { click: "Groups", file: "02-groups.png", wait: 300 },
  { click: "Docker Sync", file: "03-docker.png", wait: 300 },
  { click: "MCP Servers", file: "04-mcp.png", wait: 300 },
  { click: "LAN Access", file: "05-lan.png", wait: 300 },
  { click: "Firewall", file: "06-firewall.png", wait: 300 },
  { click: "Distros", file: "07-distros.png", wait: 300 },
  { click: "Startup Actions", file: "08-startup.png", wait: 300 },
  { click: "Boot Service", file: "09-service.png", wait: 300 },
  { click: ".wslconfig", file: "10-wslconfig.png", wait: 300 },
  { click: "Audit Log", file: "11-audit.png", wait: 300 },
  { click: "Appearance", file: "12-appearance.png", wait: 300 },
  { click: "Settings", file: "13-settings.png", wait: 300 },
];

async function main() {
  const browser = await puppeteer.launch({
    headless: true,
    args: ["--no-sandbox", "--disable-setuid-sandbox"],
    defaultViewport: { width: 1100, height: 720 },
  });

  const page = await browser.newPage();
  await page.goto(baseUrl, { waitUntil: "networkidle0" });
  await page.waitForSelector("aside"); // sidebar loaded

  for (const spec of pages) {
    // Click the sidebar nav button by text content
    const buttons = await page.$$("aside button");
    for (const btn of buttons) {
      const text = await btn.evaluate((el) => el.textContent?.trim());
      if (text === spec.click) {
        await btn.click();
        break;
      }
    }
    await new Promise((r) => setTimeout(r, spec.wait));
    await page.screenshot({ path: path.join(outDir, spec.file), type: "png" });
    console.log(`  captured: ${spec.file}`);
  }

  // Bonus: take a theme screenshot — switch to CENTCOM
  const buttons = await page.$$("aside button");
  for (const btn of buttons) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text === "Appearance") {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 300));

  // Click the CENTCOM theme button
  const themeButtons = await page.$$("main button");
  for (const btn of themeButtons) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text && text.includes("Centcom")) {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 200));

  // Navigate to rules page to show CENTCOM theme
  const navButtons2 = await page.$$("aside button");
  for (const btn of navButtons2) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text === "Port Rules") {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 300));
  await page.screenshot({
    path: path.join(outDir, "14-theme-centcom.png"),
    type: "png",
  });
  console.log("  captured: 14-theme-centcom.png");

  // Switch to Nord
  const navButtons3 = await page.$$("aside button");
  for (const btn of navButtons3) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text === "Appearance") {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 300));
  const themeButtons2 = await page.$$("main button");
  for (const btn of themeButtons2) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text && text.includes("Nord")) {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 200));
  const navButtons4 = await page.$$("aside button");
  for (const btn of navButtons4) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text === "Port Rules") {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 300));
  await page.screenshot({
    path: path.join(outDir, "15-theme-nord.png"),
    type: "png",
  });
  console.log("  captured: 15-theme-nord.png");

  // Daylight (light theme)
  const navButtons5 = await page.$$("aside button");
  for (const btn of navButtons5) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text === "Appearance") {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 300));
  const themeButtons3 = await page.$$("main button");
  for (const btn of themeButtons3) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text && text.includes("Daylight")) {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 200));
  const navButtons6 = await page.$$("aside button");
  for (const btn of navButtons6) {
    const text = await btn.evaluate((el) => el.textContent?.trim());
    if (text === "Port Rules") {
      await btn.click();
      break;
    }
  }
  await new Promise((r) => setTimeout(r, 300));
  await page.screenshot({
    path: path.join(outDir, "16-theme-daylight.png"),
    type: "png",
  });
  console.log("  captured: 16-theme-daylight.png");

  await browser.close();
  console.log(`\nDone: ${pages.length + 3} screenshots in docs/screenshots/`);
}

main().catch(console.error);
