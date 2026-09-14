// Phase 12 updater state hook
import { useSyncExternalStore } from "react";
import {
  getUpdateState,
  subscribeUpdateState,
  type UpdateState,
} from "../services/updateStore";

/**
 * Subscribes React components to the shared Phase 12 updater store.
 * Used by the Settings section, the update banner, and the install dialog.
 */
export function useUpdateState(): UpdateState {
  return useSyncExternalStore(subscribeUpdateState, getUpdateState);
}
