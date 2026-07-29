// Shared TypeScript types for the PotSpot frontend

export interface StrainSummary {
  id: string;
  name: string;
  strain_type: string;
  thc_percentage: number | null;
  cbd_percentage: number | null;
  average_rating: number | null;
  rating_count: number;
  thumbnail_url: string | null;
}

export interface StrainDetail {
  id: string;
  name: string;
  type: string;
  thc_percentage: number | null;
  cbd_percentage: number | null;
  description: string | null;
  color: string | null;
  smell: string | null;
  flavor: string | null;
  breeder: string | null;
  lineage: string | null;
  growing_difficulty: string | null;
  flowering_time_days: number | null;
  average_rating: number | null;
  rating_count: number;
  created_at: string;
  updated_at: string;
  version: number;
  terpenes: TerpeneInfo[];
  effects: EffectInfo[];
  primary_photo_url: string | null;
}

export interface TerpeneInfo {
  id: number;
  name: string;
  icon: string;
  description: string | null;
}

export interface EffectInfo {
  id: number;
  name: string;
  category: string;
}

export interface StrainListResponse {
  strains: StrainSummary[];
  total: number;
  page: number;
  per_page: number;
}
