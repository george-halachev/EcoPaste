import { Flex } from "antd";
import { filesize } from "filesize";
import type { FC } from "react";
import { useTranslation } from "react-i18next";
import type { DatabaseSchemaHistory } from "@/types/database";
import { dayjs } from "@/utils/dayjs";

interface HeaderProps {
  data: DatabaseSchemaHistory;
  handleNote: () => void;
  handleFavorite: () => void;
  handleDelete: () => void;
}

const Header: FC<HeaderProps> = (props) => {
  const { data } = props;
  const { type, value, count, createTime, subtype } = data;
  const { t, i18n } = useTranslation();

  const renderType = () => {
    switch (subtype) {
      case "url":
        return t("clipboard.label.link");
      case "email":
        return t("clipboard.label.email");
      case "color":
        return t("clipboard.label.color");
      case "path":
        return t("clipboard.label.path");
    }

    switch (type) {
      case "text":
        return t("clipboard.label.plain_text");
      case "rtf":
        return t("clipboard.label.rtf");
      case "html":
        return t("clipboard.label.html");
      case "image":
        return t("clipboard.label.image");
      case "files":
        return t("clipboard.label.n_files", {
          replace: [value.length],
        });
    }
  };

  const renderCount = () => {
    if (type === "files" || type === "image") {
      return filesize(count, { standard: "jedec" });
    }

    return t("clipboard.label.n_chars", {
      replace: [count],
    });
  };

  const renderPixel = () => {
    if (type !== "image") return;

    const { width, height } = data;

    return (
      <span>
        {width}×{height}
      </span>
    );
  };

  return (
    <Flex className="w-25 shrink-0 text-color-2" gap={2} vertical>
      <div className="flex flex-col gap-0.5 overflow-hidden text-right text-[11px] leading-tight">
        <span className="truncate">{renderType()}</span>
        <span className="truncate">{renderCount()}</span>
        {renderPixel()}
        <span className="truncate text-color-3">
          {dayjs(createTime).locale(i18n.language).fromNow()}
        </span>
      </div>
    </Flex>
  );
};

export default Header;
