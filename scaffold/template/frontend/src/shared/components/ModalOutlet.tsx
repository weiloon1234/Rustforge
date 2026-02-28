import { useCallback } from "react";
import { useModalStore } from "@shared/useModalStore";
import { Modal } from "@shared/components/Modal";

export function ModalOutlet() {
  const stack = useModalStore((s) => s.stack);
  const close = useModalStore((s) => s.close);
  const handleClose = useCallback((id: string) => close(id), [close]);

  if (stack.length === 0) return null;

  return (
    <>
      {stack.map((entry, i) => (
        <Modal key={entry.id} entry={entry} index={i} onClose={handleClose} />
      ))}
    </>
  );
}
