export interface EpisodeInfo {
  season: number;
  episode: number;
  display: string;
}

const EPISODE_PATTERNS = [
  /[Ss](\d{1,2})[Ee](\d{1,2})/,           // S01E02, s1e2
  /(\d{1,2})[xX](\d{1,2})/,                // 1x02, 01x02
  /[Ee]p?(\d{1,2})/,                       // E02, Ep02 (episode only)
  /[Ss]eason\s*(\d{1,2})\s*[Ee]pisode\s*(\d{1,2})/i,
  /(\d)(\d{2})/,                            // 102 = S1E02
];

export function parseEpisodeInfo(title: string): EpisodeInfo | null {
  const t = title.trim();
  for (const re of EPISODE_PATTERNS) {
    const m = t.match(re);
    if (m) {
      if (m.length >= 3) {
        const season = parseInt(m[1], 10);
        const episode = parseInt(m[2], 10);
        if (season >= 0 && episode >= 0) {
          return {
            season,
            episode,
            display: `S${String(season).padStart(2, "0")}E${String(episode).padStart(2, "0")}`,
          };
        }
      } else if (m.length === 2) {
        const episode = parseInt(m[1], 10);
        if (episode >= 0) {
          return {
            season: 1,
            episode,
            display: `E${String(episode).padStart(2, "0")}`,
          };
        }
      }
    }
  }
  return null;
}

export function extractSeriesName(title: string): string {
  const cleaned = title
    .replace(/[Ss]\d{1,2}[Ee]\d{1,2}/g, "")
    .replace(/\d{1,2}[xX]\d{1,2}/g, "")
    .replace(/[Ee]p?\d{1,2}/g, "")
    .replace(/[Ss]eason\s*\d{1,2}\s*[Ee]pisode\s*\d{1,2}/gi, "")
    .replace(/\./g, " ")
    .replace(/_/g, " ")
    .replace(/-/g, " ")
    .trim();

  const parts = cleaned.split(/\s+/).filter(Boolean);
  const lastNum = parts[parts.length - 1];
  if (lastNum && /^\d+$/.test(lastNum)) {
    parts.pop();
  }
  return parts.join(" ").trim() || title;
}

export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/\s+/g, "-")
    .replace(/[^a-z0-9-äöüß]/g, "")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}
