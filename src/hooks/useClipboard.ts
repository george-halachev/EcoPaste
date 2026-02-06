import { listen } from "@tauri-apps/api/event";
import { error as logError, warn as logWarn } from "@tauri-apps/plugin-log";
import { useMount } from "ahooks";
import { message } from "antd";
import { cloneDeep } from "es-toolkit";
import { isEmpty, remove } from "es-toolkit/compat";
import { nanoid } from "nanoid";
import {
  type ClipboardChangeOptions,
  COMMAND,
  type ReadClipboard,
  startListening,
} from "tauri-plugin-clipboard-x-api";
import { fullName } from "tauri-plugin-fs-pro-api";
import {
  insertHistory,
  selectHistory,
  updateHistory,
} from "@/database/history";
import { i18n } from "@/locales";
import type { State } from "@/pages/Main";
import {
  getClipboardTextSubtype,
  readClipboardWithRetry,
} from "@/plugins/clipboard";
import { clipboardStore } from "@/stores/clipboard";
import type { DatabaseSchemaHistory } from "@/types/database";
import { formatDate } from "@/utils/dayjs";

// Debounce delay to coalesce rapid clipboard change events (e.g. when apps
// set multiple formats in sequence, each triggering WM_CLIPBOARDUPDATE).
const DEBOUNCE_MS = 300;

export const useClipboard = (
  state: State,
  options?: ClipboardChangeOptions,
) => {
  useMount(async () => {
    await startListening();

    let debounceTimer: ReturnType<typeof setTimeout> | null = null;
    let isProcessing = false;

    listen(COMMAND.CLIPBOARD_CHANGED, () => {
      // Always fire beforeRead immediately so audio feedback is prompt.
      options?.beforeRead?.();

      // Debounce: reset the timer on each rapid event so we only process
      // the final clipboard state after the writing app is done.
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }

      debounceTimer = setTimeout(async () => {
        debounceTimer = null;

        // Processing lock: skip if a previous handler is still running.
        if (isProcessing) return;
        isProcessing = true;

        try {
          await processClipboard(state);
        } finally {
          isProcessing = false;
        }
      }, DEBOUNCE_MS);
    });
  });
};

async function processClipboard(state: State) {
  let result: ReadClipboard;
  try {
    result = await readClipboardWithRetry();
  } catch (err) {
    logError(`Clipboard read failed after retries: ${String(err)}`);
    message.warning(i18n.t("clipboard.hints.clipboard_read_failed"));
    return;
  }

  try {
    const { files, image, html, rtf, text } = result;

    if (isEmpty(result) || Object.values(result).every(isEmpty)) return;

    const { copyPlain } = clipboardStore.content;

    const data = {
      createTime: formatDate(),
      favorite: false,
      group: "text",
      id: nanoid(),
      search: text?.value,
    } as DatabaseSchemaHistory;

    if (files) {
      Object.assign(data, files, {
        group: "files",
        search: files.value.join(" "),
      });
    } else if (html && !copyPlain) {
      Object.assign(data, html);
    } else if (rtf && !copyPlain) {
      Object.assign(data, rtf);
    } else if (text) {
      const subtype = await getClipboardTextSubtype(text.value);

      Object.assign(data, text, {
        subtype,
      });
    } else if (image) {
      Object.assign(data, image, {
        group: "image",
      });
    } else if (copyPlain && (html || rtf)) {
      const fallbackValue = data.search || html?.value || rtf?.value || "";
      const subtype = await getClipboardTextSubtype(fallbackValue);

      Object.assign(data, {
        count: fallbackValue.length,
        subtype,
        type: "text",
        value: fallbackValue,
      });
    }

    if (!data.type) {
      logWarn("Clipboard item has no recognized type, skipping.");
      return;
    }

    // Skip 0-byte images — these occur when the clipboard is read before the
    // source app has finished writing the image data.
    if (data.type === "image" && data.count === 0) {
      logWarn("Skipping 0-byte image clipboard entry.");
      return;
    }

    const sqlData = cloneDeep(data);

    const { type, value, group, createTime } = data;

    if (type === "image") {
      sqlData.value = await fullName(value);
    }

    if (type === "files") {
      sqlData.value = JSON.stringify(value);
    }

    const [matched] = await selectHistory((qb) => {
      const { type, value } = sqlData;

      if (type === "image") {
        // For images, match by path OR by identical dimensions + file size.
        // This catches duplicates where the same image was saved to different
        // paths (e.g. due to rapid clipboard events producing different hashes
        // from partial reads).
        return qb
          .where("type", "=", "image")
          .where((eb) =>
            eb.or([
              eb("value", "=", value),
              eb.and([
                eb("width", "=", data.width),
                eb("height", "=", data.height),
                eb("count", "=", data.count),
              ]),
            ]),
          );
      }

      return qb.where("type", "=", type).where("value", "=", value);
    });

    const visible = state.group === "all" || state.group === group;

    if (matched) {
      const { id } = matched;

      if (visible) {
        remove(state.list, { id });

        state.list.unshift({ ...data, id });
      }

      return updateHistory(id, { createTime });
    }

    if (visible) {
      state.list.unshift(data);
    }

    await insertHistory(sqlData);
  } catch (err) {
    logError(`Clipboard processing failed: ${String(err)}`);
    message.warning(i18n.t("clipboard.hints.clipboard_process_failed"));
  }
}
