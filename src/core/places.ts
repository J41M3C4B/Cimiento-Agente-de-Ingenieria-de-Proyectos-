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

/**
 * Whether a page is a place of the program. A frozen module (ADR-043) is not, and nothing shows it: Rust decides which
 * ones (`core/modules.rs`), and the screens only ask. While the map loads, no module is a place.
 */
export function useIsPlace(): (id: string) => boolean {
  const places = useQuery({ queryKey: ["places"], queryFn: appPlaces, staleTime: Infinity });
  return (id) => places.data?.some((p) => p.id === id) ?? false;
}
