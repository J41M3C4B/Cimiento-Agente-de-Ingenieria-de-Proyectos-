import { useQuery } from "@tanstack/react-query";
import { appPlaces } from "../lib/tauri";

/**
 * The name of a place of the program (`profile.legal` → «Datos legales y fiscales»), as Rust composes it from the
 * manual (`core/places.rs`): what the buttons «Ir a…» say. A new page or window gets its name from its entry in the
 * manual, with nothing to write here.
 */
export function usePlaceName(): (id: string) => string {
  const places = useQuery({ queryKey: ["places"], queryFn: appPlaces, staleTime: Infinity });
  return (id) => places.data?.find((p) => p.id === id)?.label ?? "";
}
