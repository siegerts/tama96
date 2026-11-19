import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Choice, PetState } from "../types";

export interface UsePetStateReturn {
  state: PetState | null;
  loading: boolean;
  error: string | null;
  feedMeal: () => Promise<void>;
  feedSnack: () => Promise<void>;
  guessDirection: (choice: Choice, displayed: PetState) => Promise<PetState>;
  discipline: () => Promise<void>;
  giveMedicine: () => Promise<void>;
  cleanPoop: () => Promise<void>;
  toggleLights: () => Promise<void>;
  hatchNewEgg: () => Promise<void>;
}

export function usePetState(): UsePetStateReturn {
  const [state, setState] = useState<PetState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const s = await invoke<PetState>("get_state");
      if (mountedRef.current) {
        setState(current => current && current.birth_time === s.birth_time && current.game.turn > s.game.turn ? current : s);
        setError(null);
      }
    } catch (e) {
      if (mountedRef.current) {
        setError(String(e));
      }
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;

    const init = async () => {
      await refresh();
      if (mountedRef.current) {
        setLoading(false);
      }
    };
    init();

    const interval = setInterval(refresh, 1000);

    return () => {
      mountedRef.current = false;
      clearInterval(interval);
    };
  }, [refresh]);

  const feedMeal = useCallback(async () => {
    const s = await invoke<PetState>("feed_meal");
    if (mountedRef.current) setState(s);
  }, []);

  const feedSnack = useCallback(async () => {
    const s = await invoke<PetState>("feed_snack");
    if (mountedRef.current) setState(s);
  }, []);

  const guessDirection = useCallback(async (choice: Choice, displayed: PetState) => {
    try {
      const next = await invoke<PetState>("guess_direction", {
        guess: { choice, turn: displayed.game.turn, birth_time: displayed.birth_time },
      });
      if (mountedRef.current) setState(next);
      return next;
    } catch (e) {
      await refresh();
      throw e;
    }
  }, [refresh]);

  const discipline = useCallback(async () => {
    const s = await invoke<PetState>("discipline");
    if (mountedRef.current) setState(s);
  }, []);

  const giveMedicine = useCallback(async () => {
    const s = await invoke<PetState>("give_medicine");
    if (mountedRef.current) setState(s);
  }, []);

  const cleanPoop = useCallback(async () => {
    const s = await invoke<PetState>("clean_poop");
    if (mountedRef.current) setState(s);
  }, []);

  const toggleLights = useCallback(async () => {
    const s = await invoke<PetState>("toggle_lights");
    if (mountedRef.current) setState(s);
  }, []);

  const hatchNewEgg = useCallback(async () => {
    const s = await invoke<PetState>("hatch_new_egg");
    if (mountedRef.current) setState(s);
  }, []);

  return {
    state,
    loading,
    error,
    feedMeal,
    feedSnack,
    guessDirection,
    discipline,
    giveMedicine,
    cleanPoop,
    toggleLights,
    hatchNewEgg,
  };
}
