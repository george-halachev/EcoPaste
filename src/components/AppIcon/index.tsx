import type { CSSProperties, FC } from "react";
import { useSnapshot } from "valtio";
import { settingsState } from "@/stores/settings";
import type { IconStyle } from "@/types/settings";
import { cn } from "@/utils/cn";

interface AppIconProps {
  alt?: string;
  className?: string;
  iconStyle?: IconStyle;
}

/** Renders the selected app identity with the surrounding theme's text color. */
const AppIcon: FC<AppIconProps> = (props) => {
  const { alt = "EcoPaste", className, iconStyle } = props;
  const settings = useSnapshot(settingsState);
  const selected = iconStyle ?? settings.appearance?.iconStyle ?? "outline";
  const style: CSSProperties = {
    maskImage: `url("/icons/ecopaste-${selected}.svg")`,
    WebkitMaskImage: `url("/icons/ecopaste-${selected}.svg")`,
  };

  return (
    <span
      aria-hidden={alt.length === 0 ? true : void 0}
      aria-label={alt.length > 0 ? alt : void 0}
      className={cn(
        "mask-contain mask-center mask-no-repeat inline-block size-5 shrink-0 bg-current",
        className,
      )}
      data-app-icon={selected}
      role="img"
      style={style}
    />
  );
};

export default AppIcon;
