import { useState } from "react";
import {
  Globe,
  Database,
  Cpu,
  Package,
  Wrench,
  Check,
  ChevronDown,
  ChevronRight,
  Download,
  AlertCircle,
} from "lucide-react";
import type { CategoryGroup, ComponentInfo } from "@/types";
import { formatInstallerBytes } from "@/types/installer";

interface ComponentListProps {
  categoryGroups: CategoryGroup[];
  selectedIds: string[];
  onToggle: (componentId: string) => void;
  onInstall?: (componentId: string) => void;
  showInstallButton?: boolean;
}

const CATEGORY_ICONS: Record<string, React.ElementType> = {
  WebServer: Globe,
  Database: Database,
  Runtime: Cpu,
  PackageManager: Package,
  DevTool: Wrench,
};

const CATEGORY_COLORS: Record<string, string> = {
  WebServer: "text-blue-400",
  Database: "text-green-400",
  Runtime: "text-yellow-400",
  PackageManager: "text-cyan-400",
  DevTool: "text-pink-400",
};

export function ComponentList({
  categoryGroups,
  selectedIds,
  onToggle,
  onInstall,
  showInstallButton = false,
}: ComponentListProps) {
  const [expandedCategories, setExpandedCategories] = useState<string[]>(
    categoryGroups.map((g) => g.category)
  );

  const toggleCategory = (category: string) => {
    setExpandedCategories((prev) =>
      prev.includes(category)
        ? prev.filter((c) => c !== category)
        : [...prev, category]
    );
  };

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold text-zinc-100">
        추가로 설치할 구성요소가 있다면 선택하세요
      </h2>

      <div className="space-y-2">
        {categoryGroups.map((group) => {
          const Icon = CATEGORY_ICONS[group.category] || Package;
          const colorClass = CATEGORY_COLORS[group.category] || "text-zinc-400";
          const isExpanded = expandedCategories.includes(group.category);
          const selectedInCategory = group.components.filter((c) =>
            selectedIds.includes(c.id)
          ).length;

          return (
            <div
              key={group.category}
              className="border border-zinc-700 rounded-lg overflow-hidden"
            >
              <button
                onClick={() => toggleCategory(group.category)}
                className="w-full px-4 py-3 flex items-center justify-between bg-zinc-800/50 hover:bg-zinc-800 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <Icon className={`w-5 h-5 ${colorClass}`} />
                  <span className="font-medium text-zinc-200">
                    {group.displayName}
                  </span>
                  {selectedInCategory > 0 && (
                    <span className="px-2 py-0.5 text-xs rounded-full bg-blue-500/20 text-blue-400">
                      {selectedInCategory} 선택됨
                    </span>
                  )}
                </div>
                {isExpanded ? (
                  <ChevronDown className="w-4 h-4 text-zinc-500" />
                ) : (
                  <ChevronRight className="w-4 h-4 text-zinc-500" />
                )}
              </button>

              {isExpanded && (
                <div className="divide-y divide-zinc-700/50">
                  {group.components.map((component) => (
                    <ComponentRow
                      key={component.id}
                      component={component}
                      isSelected={selectedIds.includes(component.id)}
                      onToggle={() => onToggle(component.id)}
                      onInstall={onInstall}
                      showInstallButton={showInstallButton}
                    />
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

interface ComponentRowProps {
  component: ComponentInfo;
  isSelected: boolean;
  onToggle: () => void;
  onInstall?: (componentId: string) => void;
  showInstallButton: boolean;
}

function ComponentRow({
  component,
  isSelected,
  onToggle,
  onInstall,
  showInstallButton,
}: ComponentRowProps) {
  const isDisabled = component.isInstalled;

  const sourceLabel = component.installedSource
    ? ` (${component.installedSource})`
    : "";

  return (
    <div className={`px-4 py-3 flex items-center justify-between transition-colors ${
      isDisabled ? "opacity-60" : "hover:bg-zinc-800/30"
    }`}>
      <div className="flex items-center gap-3 flex-1">
        <button
          onClick={isDisabled ? undefined : onToggle}
          disabled={isDisabled}
          className={`w-5 h-5 rounded border-2 flex items-center justify-center transition-colors ${
            isDisabled
              ? "border-zinc-600 bg-zinc-700 cursor-not-allowed"
              : isSelected
                ? "border-blue-500 bg-blue-500"
                : "border-zinc-600 hover:border-zinc-500"
          }`}
        >
          {(isSelected || isDisabled) && <Check className={`w-3 h-3 ${isDisabled ? "text-zinc-500" : "text-white"}`} />}
        </button>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className={`font-medium ${isDisabled ? "text-zinc-400" : "text-zinc-200"}`}>{component.name}</span>
            <span className="text-xs text-zinc-500">{component.version}</span>
            {component.isInstalled && (
              <span className="px-1.5 py-0.5 text-xs rounded bg-green-500/20 text-green-400">
                설치됨{sourceLabel}
              </span>
            )}
            {component.isInstalled && component.installedVersion && (
              <span className="text-xs text-zinc-500">
                v{component.installedVersion}
              </span>
            )}
            {!component.isInstalled && !component.hasBundle && (
              <span className="px-1.5 py-0.5 text-xs rounded bg-amber-500/20 text-amber-400 flex items-center gap-1">
                <Download className="w-3 h-3" />
                다운로드 필요
              </span>
            )}
          </div>
          <p className="text-xs text-zinc-500 truncate">
            {component.description}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3 ml-4">
        <span className="text-xs text-zinc-500">
          {formatInstallerBytes(component.sizeBytes)}
        </span>

        {showInstallButton && !component.isInstalled && onInstall && (
          <button
            onClick={() => onInstall(component.id)}
            className="px-3 py-1.5 text-xs rounded bg-blue-500 hover:bg-blue-600 text-white transition-colors"
          >
            설치하기
          </button>
        )}

        {component.dependencies.length > 0 && (
          <div
            className="text-zinc-500"
            title={`의존성: ${component.dependencies.join(", ")}`}
          >
            <AlertCircle className="w-4 h-4" />
          </div>
        )}
      </div>
    </div>
  );
}
