import { exists } from "@tauri-apps/plugin-fs";
import { error as logError, warn as logWarn } from "@tauri-apps/plugin-log";
import {
  hasFiles,
  hasHTML,
  hasImage,
  hasRTF,
  hasText,
  type ReadClipboard,
  readFiles,
  readHTML,
  readImage,
  readRTF,
  readText,
  writeFiles,
  writeHTML,
  writeImage,
  writeRTF,
  writeText,
} from "tauri-plugin-clipboard-x-api";
import { clipboardStore } from "@/stores/clipboard";
import type { DatabaseSchemaHistory } from "@/types/database";
import { isColor, isEmail, isURL } from "@/utils/is";
import { paste } from "./paste";

export const getClipboardTextSubtype = async (value: string) => {
  try {
    if (isURL(value)) {
      return "url";
    }

    if (isEmail(value)) {
      return "email";
    }

    if (isColor(value)) {
      return "color";
    }

    if (await exists(value)) {
      return "path";
    }
  } catch {
    return;
  }
};

export const writeToClipboard = (data: DatabaseSchemaHistory) => {
  const { type, value, search } = data;

  switch (type) {
    case "text":
      return writeText(value);
    case "rtf":
      return writeRTF(search, value);
    case "html":
      return writeHTML(search, value);
    case "image":
      return writeImage(value);
    case "files":
      return writeFiles(value);
  }
};

export const pasteToClipboard = async (
  data: DatabaseSchemaHistory,
  asPlain?: boolean,
) => {
  const { type, value, search } = data;
  const { pastePlain } = clipboardStore.content;

  if (asPlain ?? pastePlain) {
    if (type === "files") {
      await writeText(value.join("\n"));
    } else {
      await writeText(search);
    }
  } else {
    await writeToClipboard(data);
  }

  return paste();
};

const RETRY_COUNT = 3;
const RETRY_DELAY_MS = 200;

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Reads all available clipboard content with per-format error isolation
 * and retry logic for transient clipboard lock failures (common on Windows).
 */
export const readClipboardWithRetry = async (): Promise<ReadClipboard> => {
  let lastError: unknown;

  for (let attempt = 1; attempt <= RETRY_COUNT; attempt++) {
    try {
      const result: ReadClipboard = {};

      try {
        if (await hasText()) {
          const text = await readText();
          result.text = { count: text.length, type: "text", value: text };
        }
      } catch (err) {
        logWarn(
          `readClipboard: failed to read text (attempt ${attempt}): ${String(err)}`,
        );
        throw err;
      }

      try {
        if (await hasRTF()) {
          const rtf = await readRTF();
          result.rtf = {
            count: result.text?.count ?? rtf.length,
            type: "rtf",
            value: rtf,
          };
        }
      } catch (err) {
        logWarn(
          `readClipboard: failed to read RTF (attempt ${attempt}): ${String(err)}`,
        );
        if (!result.text) throw err;
      }

      try {
        if (await hasHTML()) {
          const html = await readHTML();
          result.html = {
            count: result.text?.count ?? html.length,
            type: "html",
            value: html,
          };
        }
      } catch (err) {
        logWarn(
          `readClipboard: failed to read HTML (attempt ${attempt}): ${String(err)}`,
        );
        if (!result.text) throw err;
      }

      try {
        if (await hasImage()) {
          const { path, size, ...rest } = await readImage();
          result.image = {
            count: size,
            type: "image",
            value: path,
            ...rest,
          };
        }
      } catch (err) {
        logWarn(
          `readClipboard: failed to read image (attempt ${attempt}): ${String(err)}`,
        );
        if (!result.text && !result.html && !result.rtf) throw err;
      }

      try {
        if (await hasFiles()) {
          const { paths, size } = await readFiles();
          result.files = { count: size, type: "files", value: paths };
        }
      } catch (err) {
        logWarn(
          `readClipboard: failed to read files (attempt ${attempt}): ${String(err)}`,
        );
        if (!result.text && !result.html && !result.rtf && !result.image)
          throw err;
      }

      return result;
    } catch (err) {
      lastError = err;
      if (attempt < RETRY_COUNT) {
        await delay(RETRY_DELAY_MS);
      }
    }
  }

  logError(
    `readClipboard: all ${RETRY_COUNT} attempts failed: ${String(lastError)}`,
  );
  throw lastError;
};
