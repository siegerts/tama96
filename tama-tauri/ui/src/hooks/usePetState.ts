import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Choice, GameResult, PetState } from "../types";

export interface UsePetStateReturn {
  state: PetState | null;
  loading: boolean;
  error: string | null;
  feedMeal: () => Promise<void>;
  feedSnack: () => Promise<void>;
  playGame: (moves: Choice[]) => Promise<GameResult>;
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
        setState(s);
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
    try {
      const s = await invoke<PetState>("feed_meal");
      console.log("feed_meal result:", s);
      if (mountedRef.current) setState(s);
    } catch (e) {
      console.error("feed_meal error:", e);
    }
  }, []);

  const feedSnack = useCallback(async () => {
    try {
      const s = await invoke<PetState>("feed_snack");
      console.log("feed_snack result:", s);
      if (mountedRef.current) setState(s);
    } catch (e) {
      console.error("feed_snack error:", e);
    }
  }, []);

  const playGame = useCallback(async (moves: Choice[]): Promise<GameResult> => {
    const result = await invoke<GameResult>("play_game", { moves });
    await refresh();
    return result;
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
    playGame,
    discipline,
    giveMedicine,
    cleanPoop,
    toggleLights,
    hatchNewEgg,
  };
}
