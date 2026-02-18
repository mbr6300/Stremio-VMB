import type { LibraryItemWithMeta, MovieGroup } from "../types";

function normalizeWhitespace(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function normalizeMovieTitle(value: string): string {
  const cleaned = value
    .toLowerCase()
    .replace(/\.[a-z0-9]{2,4}$/i, "")
    .replace(/[._-]+/g, " ")
    .replace(/\b(2160p|1080p|720p|480p|4k|8k|uhd|hdr|x264|x265|hevc|bluray|webrip|web-dl|dvdrip)\b/gi, "")
    .replace(/[()[\]{}]/g, " ");
  return normalizeWhitespace(cleaned);
}

function normalizePosterUrl(value: string): string {
  return value.split("?")[0].trim();
}

function selectRepresentative(entries: LibraryItemWithMeta[]): LibraryItemWithMeta {
  const sorted = [...entries].sort((a, b) => {
    const aScore = (a.metadata ? 1 : 0) * 1_000_000_000_000 + (a.item.file_size ?? 0);
    const bScore = (b.metadata ? 1 : 0) * 1_000_000_000_000 + (b.item.file_size ?? 0);
    return bScore - aScore;
  });
  return sorted[0];
}

export function buildMovieGroups(items: LibraryItemWithMeta[]): MovieGroup[] {
  const movies = items.filter((entry) => entry.item.media_type === "movie");
  if (movies.length === 0) return [];

  const parent = Array.from({ length: movies.length }, (_, i) => i);
  const titleIndex = new Map<string, number>();
  const posterIndex = new Map<string, number>();

  function find(i: number): number {
    if (parent[i] !== i) parent[i] = find(parent[i]);
    return parent[i];
  }

  function union(a: number, b: number) {
    const rootA = find(a);
    const rootB = find(b);
    if (rootA !== rootB) parent[rootB] = rootA;
  }

  for (let i = 0; i < movies.length; i += 1) {
    const current = movies[i];
    const titleKey = normalizeMovieTitle(current.metadata?.title ?? current.item.title);
    if (titleKey) {
      const existing = titleIndex.get(titleKey);
      if (existing != null) union(existing, i);
      else titleIndex.set(titleKey, i);
    }

    const posterRaw = current.metadata?.poster_url?.trim();
    if (posterRaw) {
      const posterKey = normalizePosterUrl(posterRaw);
      const existing = posterIndex.get(posterKey);
      if (existing != null) union(existing, i);
      else posterIndex.set(posterKey, i);
    }
  }

  const grouped = new Map<number, LibraryItemWithMeta[]>();
  for (let i = 0; i < movies.length; i += 1) {
    const root = find(i);
    const list = grouped.get(root) ?? [];
    list.push(movies[i]);
    grouped.set(root, list);
  }

  return Array.from(grouped.entries()).map(([root, entries]) => {
    const representative = selectRepresentative(entries);
    return {
      key: `movie-group-${root}-${representative.item.id}`,
      representative,
      entries,
    };
  });
}
