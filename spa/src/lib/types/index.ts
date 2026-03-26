export interface Track {
  tid: number;
  title: string;
  uid: number;
  username: string;
  is_visible: boolean;
  is_hidden: boolean;
  date: string;
  timestamp: string;
  day: string;
  has_art: boolean;
}

export interface ExtendedTrack extends Track {
  email_md5: string;
  notes: string;
  notes_html: string;
  has_notes: boolean;
  license: string;
  has_license: boolean;
  tags: string[];
  has_tags: boolean;
  is_copyright: boolean;
  license_key: string;
  airable: boolean;
}

export interface UserProfile {
  uid: number;
  username: string;
  email_md5: string;
  about: string;
  about_html: string;
  has_about: boolean;
  num_favs: number;
  has_favs: boolean;
  num_followers: number;
  has_followers: boolean;
  tracks: Track[];
  playlists: PlaylistSummary[];
  albums: AlbumSummary[];
  events: EventItem[];
}

export interface PlaylistSummary {
  playlist_id: number;
  playlist_name: string;
  playlist_url: string;
  track_count: string;
  playlist_track_count: number;
  uid: number;
  username: string;
  description: string;
  description_html: string;
  has_description: boolean;
  is_public: boolean;
  is_album: boolean;
}

export interface PlaylistData {
  playlist: PlaylistSummary;
  tracks: Track[];
}

// Albums use the same shape as playlists (they are playlists with is_album=true)
export type AlbumSummary = PlaylistSummary;
export type AlbumData = PlaylistData;

export interface EventItem {
  event_id: number;
  utc_date: string;
  fuzzy_time: string;
  is_publish: boolean;
  is_comment: boolean;
  is_favorite: boolean;
  is_follow: boolean;
  source_uid: number;
  source_name: string;
  target_uid: number;
  target_name: string;
  has_track: boolean;
  tid: number;
  track_title: string;
  message: string;
  message_html: string;
}

export interface MeResponse {
  logged_in: boolean;
  uid?: number;
  username?: string;
}

export interface Artist {
  uid: number;
  username: string;
  email_md5: string;
  about_html: string;
  has_about: boolean;
}

export interface AudioStatus {
  ready: boolean;
  status: string;
  has_status: boolean;
  is_mp3_source: boolean;
  is_other_source: boolean;
  extension: string;
}

export interface ApiError {
  error: string;
  message: string;
}

/**
 * Describes the playback context so the player can fetch more tracks.
 * context: "latest" | "featured" | "random" | "search" | "tag" |
 *          "playlist" | "user" | "favorites"
 * param:   extra info depending on context (query string, tag, id, etc.)
 */
export interface PlaybackContext {
  context: string;
  param?: string;
}
