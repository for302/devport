import { Hexagon, Code, Layers, Check } from "lucide-react";
import type { PresetInfo } from "@/types";
import { formatInstallerBytes } from "@/types/installer";

interface PresetSelectorProps {
  presets: PresetInfo[];
  selectedId: string | null;
  onSelect: (presetId: string) => void;
}

const PRESET_ICONS: Record<string, React.ElementType> = {
  hexagon: Hexagon,
  code: Code,
  layers: Layers,
};

// 프리셋 표시 순서: node -> php -> all
const PRESET_ORDER: Record<string, number> = {
  node: 0,
  php: 1,
  all: 2,
};

export function PresetSelector({
  presets,
  selectedId,
  onSelect,
}: PresetSelectorProps) {
  // 프리셋을 정해진 순서로 정렬
  const sortedPresets = [...presets].sort((a, b) => {
    const orderA = PRESET_ORDER[a.id] ?? 99;
    const orderB = PRESET_ORDER[b.id] ?? 99;
    return orderA - orderB;
  });

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold text-zinc-100">
        개발 환경을 선택하세요
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {sortedPresets.map((preset) => {
          const Icon = PRESET_ICONS[preset.icon] || Hexagon;
          const isSelected = selectedId === preset.id;

          return (
            <button
              key={preset.id}
              onClick={() => onSelect(preset.id)}
              className={`relative p-6 rounded-lg border-2 transition-all text-left ${
                isSelected
                  ? "border-blue-500 bg-blue-500/10"
                  : "border-zinc-700 bg-zinc-800/50 hover:border-zinc-600"
              }`}
            >
              {isSelected && (
                <div className="absolute top-3 right-3">
                  <Check className="w-5 h-5 text-blue-400" />
                </div>
              )}

              <div className="flex items-center gap-3 mb-3">
                <div
                  className={`p-2 rounded-lg ${
                    isSelected ? "bg-blue-500/20" : "bg-zinc-700"
                  }`}
                >
                  <Icon
                    className={`w-6 h-6 ${
                      isSelected ? "text-blue-400" : "text-zinc-400"
                    }`}
                  />
                </div>
                <h3 className="font-semibold text-zinc-100">{preset.name}</h3>
              </div>

              <p className="text-sm text-zinc-400 mb-4">{preset.description}</p>

              <div className="space-y-2">
                <div className="flex flex-wrap gap-1">
                  {preset.components.slice(0, 4).map((comp) => (
                    <span
                      key={comp}
                      className="px-2 py-0.5 text-xs rounded bg-zinc-700 text-zinc-300"
                    >
                      {comp}
                    </span>
                  ))}
                  {preset.components.length > 4 && (
                    <span className="px-2 py-0.5 text-xs rounded bg-zinc-700 text-zinc-400">
                      +{preset.components.length - 4}
                    </span>
                  )}
                </div>

                <p className="text-xs text-zinc-500">
                  예상 용량: {formatInstallerBytes(preset.totalSizeBytes)}
                </p>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
