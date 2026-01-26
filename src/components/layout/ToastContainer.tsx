import { X, Info, CheckCircle, AlertTriangle, XCircle } from "lucide-react";
import { useUiStore, type Notification, type NotificationType } from "@/stores/uiStore";

const iconMap: Record<NotificationType, typeof Info> = {
  info: Info,
  success: CheckCircle,
  warning: AlertTriangle,
  error: XCircle,
};

const bgColorMap: Record<NotificationType, string> = {
  info: "bg-blue-600",
  success: "bg-green-600",
  warning: "bg-yellow-600",
  error: "bg-red-600",
};

function Toast({ notification }: { notification: Notification }) {
  const removeNotification = useUiStore((state) => state.removeNotification);
  const Icon = iconMap[notification.type];
  const bgColor = bgColorMap[notification.type];

  return (
    <div
      className={`${bgColor} rounded-lg shadow-lg p-4 min-w-[300px] max-w-[400px] flex items-start gap-3 animate-slide-in`}
    >
      <Icon className="w-5 h-5 text-white flex-shrink-0 mt-0.5" />
      <div className="flex-1 min-w-0">
        <h4 className="text-white font-medium text-sm">{notification.title}</h4>
        {notification.message && (
          <p className="text-white/80 text-xs mt-1">{notification.message}</p>
        )}
      </div>
      <button
        onClick={() => removeNotification(notification.id)}
        className="text-white/60 hover:text-white transition-colors"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}

export function ToastContainer() {
  const notifications = useUiStore((state) => state.notifications);

  if (notifications.length === 0) {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
      {notifications.map((notification) => (
        <Toast key={notification.id} notification={notification} />
      ))}
    </div>
  );
}
