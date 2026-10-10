import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const output = "src-tauri/assets/app-icons";
const scratch = "src-tauri/target/icon-generation";
const sizes = [16, 24, 32, 48, 64, 128, 256];
const palettes = { dark: "#f3f3f3", light: "#242424", neutral: "#737373" };

/** Builds a multi-resolution ICO from the rendered PNGs, preserving their alpha. */
function writeIco(name: string) {
  const images = sizes.map((size) => {
    return readFileSync(join(output, `${name}-${size}.png`));
  });
  const header = Buffer.alloc(6 + images.length * 16);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(images.length, 4);
  let offset = header.length;

  images.forEach((png, index) => {
    const entry = 6 + index * 16;
    header[entry] = sizes[index] % 256;
    header[entry + 1] = sizes[index] % 256;
    header.writeUInt16LE(1, entry + 4);
    header.writeUInt16LE(32, entry + 6);
    header.writeUInt32LE(png.length, entry + 8);
    header.writeUInt32LE(offset, entry + 12);
    offset += png.length;
  });

  writeFileSync(
    join(output, `${name}.ico`),
    Buffer.concat([header, ...images]),
  );
}

mkdirSync(output, { recursive: true });
mkdirSync(scratch, { recursive: true });

for (const style of ["outline", "solid"]) {
  const source = readFileSync(`public/icons/ecopaste-${style}.svg`, "utf8");

  for (const [palette, color] of Object.entries(palettes)) {
    const name = `${style}-${palette}`;
    const svg = join(scratch, `${name}.svg`);
    const rendered = join(scratch, name);
    writeFileSync(svg, source.replaceAll("currentColor", color));
    execFileSync(
      process.execPath,
      [
        "node_modules/@tauri-apps/cli/tauri.js",
        "icon",
        svg,
        "--output",
        rendered,
        ...sizes.flatMap((size) => {
          return ["--png", String(size)];
        }),
      ],
      { stdio: "inherit" },
    );

    for (const size of sizes) {
      copyFileSync(
        join(rendered, `${size}x${size}.png`),
        join(output, `${name}-${size}.png`),
      );
    }

    writeIco(name);

    if (palette === "neutral") {
      copyFileSync(
        join(output, `${name}-256.png`),
        `public/icons/ecopaste-${style}-neutral.png`,
      );
    }
  }
}

// Windows embedded icons need contrast on both light Explorer and dark taskbars.
copyFileSync(
  join(output, "outline-neutral-32.png"),
  "src-tauri/icons/32x32.png",
);
copyFileSync(
  join(output, "outline-neutral-128.png"),
  "src-tauri/icons/128x128.png",
);
copyFileSync(
  join(output, "outline-neutral-256.png"),
  "src-tauri/icons/128x128@2x.png",
);
copyFileSync(join(output, "outline-neutral.ico"), "src-tauri/icons/icon.ico");
copyFileSync(join(output, "outline-neutral.ico"), "public/favicon.ico");

execFileSync(
  process.execPath,
  [
    "node_modules/@tauri-apps/cli/tauri.js",
    "icon",
    join(scratch, "outline-neutral.svg"),
    "--output",
    join(scratch, "bundle"),
  ],
  { stdio: "inherit" },
);
copyFileSync(join(scratch, "bundle/icon.icns"), "src-tauri/icons/icon.icns");
