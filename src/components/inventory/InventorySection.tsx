import { useState } from "react";
import { ChevronDown, ChevronRight, type LucideIcon } from "lucide-react";
import type { InventoryItem, ServiceInfo, InventoryCategory } from "@/types";
import { INVENTORY_TO_SERVICE_MAP, CONTROLLABLE_CATEGORIES } from "@/types";
import { InventoryItemRow } from "./InventoryItemRow";

interface InventorySectionProps {
  title: string;
  icon: LucideIcon;
  iconColor: string;
  items: InventoryItem[];
  category: InventoryCategory;
  showInstalledOnly: boolean;
  searchQuery: string;
  defaultExpanded?: boolean;
  services: ServiceInfo[];
  loadingServiceId: string | null;
  loadingAction: "start" | "stop" | "restart" | null;
  onStartService: (serviceId: string) => void;
  onStopService: (serviceId: string) => void;
  onRestartService: (serviceId: string) => void;
  onViewLogs: (serviceId: string, serviceName: string) => void;
  onSettings: (serviceId: string) => void;
  onInstallItem?: (itemId: string, itemName: string) => void;
}

export function InventorySection({
  title,
  icon: Icon,
  iconColor,
  items,
  category,
  showInstalledOnly,
  searchQuery,
  defaultExpanded = true,
  services,
  loadingServiceId,
  loadingAction,
  onStartService,
  onStopService,
  onRestartService,
  onViewLogs,
  onSettings,
  onInstallItem,
}: InventorySectionProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  const isControllableCategory = CONTROLLABLE_CATEGORIES.includes(category);

  // Get service for an inventory item
  const getServiceForItem = (item: InventoryItem): ServiceInfo | undefined => {
    if (!isControllableCategory) return undefined;
    const serviceId = INVENTORY_TO_SERVICE_MAP[item.id];
    if (!serviceId) return undefined;
    return services.find((s) => s.id === serviceId);
  };

  // Filter items based on filters
  const filteredItems = items.filter((item) => {
    if (showInstalledOnly && !item.isInstalled) return false;
    if (
      searchQuery &&
      !item.name.toLowerCase().includes(searchQuery.toLowerCase())
    )
      return false;
    return true;
  });

  const installedCount = items.filter((i) => i.isInstalled).length;
  const totalCount = items.length;

  // Don't render the section if no items match the filter
  if (filteredItems.length === 0 && (showInstalledOnly || searchQuery)) {
    return null;
  }

  return (
    <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden mb-4">
      {/* Header */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between px-4 py-3 bg-slate-900 hover:bg-slate-800/80 transition-colors"
      >
        <div className="flex items-center gap-3">
          <Icon size={20} className={iconColor} />
          <span className="font-medium text-white">{title}</span>
          <span className="text-sm text-slate-400">
            ({installedCount}/{totalCount} installed)
          </span>
        </div>
        {isExpanded ? (
          <ChevronDown size={18} className="text-slate-400" />
        ) : (
          <ChevronRight size={18} className="text-slate-400" />
        )}
      </button>

      {/* Content */}
      {isExpanded && (
        <div>
          {/* Column headers */}
          <div className="flex items-center gap-3 px-4 py-2 bg-slate-900/50 border-b border-slate-700 text-xs text-slate-500 uppercase tracking-wider">
            <div className="w-2 flex-shrink-0" />
            <div className="w-32 min-w-[8rem] flex-shrink-0">Name</div>
            <div className="w-20 text-center flex-shrink-0">Version</div>
            <div className="w-20 text-center flex-shrink-0">Source</div>
            <div className="w-14 text-center flex-shrink-0">Port</div>
            <div className="w-20 text-center flex-shrink-0">Status</div>
            <div className="flex-1" />
            {isControllableCategory && (
              <div className="w-40 text-center flex-shrink-0">Actions</div>
            )}
            {!isControllableCategory && <div className="w-32 flex-shrink-0" />}
          </div>

          {/* Items */}
          {filteredItems.length > 0 ? (
            filteredItems.map((item) => {
              const service = getServiceForItem(item);
              const serviceId = INVENTORY_TO_SERVICE_MAP[item.id];

              return (
                <InventoryItemRow
                  key={item.id}
                  item={item}
                  service={service}
                  isLoading={serviceId === loadingServiceId}
                  loadingAction={serviceId === loadingServiceId ? loadingAction : null}
                  onStart={serviceId ? () => onStartService(serviceId) : undefined}
                  onStop={serviceId ? () => onStopService(serviceId) : undefined}
                  onRestart={serviceId ? () => onRestartService(serviceId) : undefined}
                  onViewLogs={
                    serviceId && service
                      ? () => onViewLogs(serviceId, service.name)
                      : undefined
                  }
                  onSettings={serviceId ? () => onSettings(serviceId) : undefined}
                  onInstall={
                    !item.isInstalled && onInstallItem
                      ? () => onInstallItem(item.id, item.name)
                      : undefined
                  }
                />
              );
            })
          ) : (
            <div className="py-6 text-center text-slate-500">
              No items in this category
            </div>
          )}
        </div>
      )}
    </div>
  );
}
