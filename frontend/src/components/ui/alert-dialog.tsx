import * as React from "react"
import { cn } from "@/lib/utils"

interface AlertDialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  description: string;
  cancelText?: string;
  confirmText?: string;
  onConfirm?: () => void;
}

export function AlertDialog({
  open,
  onClose,
  title,
  description,
  cancelText = "Cancel",
  confirmText = "Confirm",
  onConfirm,
}: AlertDialogProps) {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black/50" onClick={onClose} />
      <div className="relative bg-white rounded-lg p-6 max-w-md w-full mx-4">
        <h2 className="text-lg font-semibold mb-2">{title}</h2>
        <p className="text-gray-600 mb-6">{description}</p>
        <div className="flex justify-end space-x-4">
          <button
            className={cn(
              "px-4 py-2 rounded-md text-sm font-medium",
              "bg-gray-100 text-gray-700 hover:bg-gray-200"
            )}
            onClick={onClose}
          >
            {cancelText}
          </button>
          {onConfirm && (
            <button
              className={cn(
                "px-4 py-2 rounded-md text-sm font-medium",
                "bg-blue-500 text-white hover:bg-blue-600"
              )}
              onClick={() => {
                onConfirm();
                onClose();
              }}
            >
              {confirmText}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}