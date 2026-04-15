// Workspace types
export interface Workspace {
  id: string
  name: string
  icon: string | null
  color: string | null
  db_file: string
  sort_order: number
  created_at: string
  updated_at: string
}

// Bookmark types
export interface BookmarkGroup {
  id: string
  name: string
  icon: string | null
  color: string | null
  parent_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface Bookmark {
  id: string
  title: string
  url: string
  description: string | null
  favicon_url: string | null
  group_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface CreateBookmark {
  title: string
  url: string
  description?: string
  favicon_url?: string
  group_id?: string
}

export interface UpdateBookmark {
  title?: string
  url?: string
  description?: string
  favicon_url?: string
  group_id?: string
  sort_order?: number
}

// Account types
export interface AccountGroup {
  id: string
  name: string
  icon: string | null
  color: string | null
  parent_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface Account {
  id: string
  title: string
  url: string | null
  username: string | null
  password: string
  notes: string | null
  icon_url: string | null
  group_id: string | null
  favorite: number
  sort_order: number
  created_at: string
  updated_at: string
}

export interface CreateAccount {
  title: string
  url?: string
  username?: string
  password: string
  notes?: string
  icon_url?: string
  group_id?: string
}

export interface UpdateAccount {
  title?: string
  url?: string
  username?: string
  password?: string
  notes?: string
  icon_url?: string
  group_id?: string
  favorite?: number
  sort_order?: number
}

export interface PasswordGenOptions {
  length?: number
  uppercase?: boolean
  lowercase?: boolean
  numbers?: boolean
  symbols?: boolean
  exclude_ambiguous?: boolean
}

// Note types
export interface NoteGroup {
  id: string
  name: string
  icon: string | null
  color: string | null
  parent_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface Note {
  id: string
  title: string
  content: string
  is_encrypted: number
  encrypted_content: string | null
  encryption_nonce: string | null
  group_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface NoteSummary {
  id: string
  title: string
  is_encrypted: number
  group_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface CreateNote {
  title: string
  content: string
  group_id?: string
}

export interface UpdateNote {
  title?: string
  content?: string
  group_id?: string
  sort_order?: number
}

// Report types
export interface Report {
  id: string
  title: string
  type: 'daily' | 'weekly' | 'monthly'
  content: string
  date: string
  week_start: string | null
  week_end: string | null
  month: string | null
  created_at: string
  updated_at: string
}

export interface CreateReport {
  title: string
  type: 'daily' | 'weekly' | 'monthly'
  content: string
  date: string
  week_start?: string
  week_end?: string
  month?: string
}

export interface UpdateReport {
  title?: string
  content?: string
  date?: string
  week_start?: string
  week_end?: string
  month?: string
}

// Media types
export interface MediaAsset {
  id: string
  filename: string
  mime_type: string
  size: number
  storage_type: 'local' | 's3'
  storage_path: string
  thumbnail_path: string | null
  description: string | null
  metadata: string | null
  s3_bucket: string | null
  s3_region: string | null
  s3_endpoint: string | null
  /** Absolute OS path for local assets — use with convertFileSrc() */
  absolute_path: string | null
  created_at: string
  updated_at: string
}

export interface UploadMediaPayload {
  file_path: string
  storage_type: 'local' | 's3'
  description?: string
}

export interface S3Config {
  provider: 'aws' | 'minio' | 'cloudflare' | 'custom'
  endpoint?: string
  region: string
  bucket: string
  access_key_id: string
  secret_access_key: string
  path_prefix?: string
  public_url?: string
}

// Tag types
export interface Tag {
  id: string
  name: string
  color: string | null
  icon: string | null
  created_at: string
  usage_count: number
}

export interface Taggable {
  tag_id: string
  taggable_type: 'bookmark' | 'account' | 'note' | 'report' | 'media'
  taggable_id: string
  created_at: string
}

// Search types
export interface SearchResult {
  entity_type: 'bookmark' | 'account' | 'note' | 'report' | 'media'
  entity_id: string
  title: string
  content: string
  tags: string
}

export interface SearchFilters {
  entity_types?: string[]
}

// Encrypted note payload
export interface EncryptNotePayload {
  id: string
  secondary_password: string
}

export interface DecryptNotePayload {
  id: string
  secondary_password: string
}

// Group type (polymorphic)
export interface AnyGroup {
  id: string
  name: string
  icon: string | null
  color: string | null
  parent_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
  type: 'bookmark' | 'account' | 'note'
}

// Generic create/update payloads
export interface CreateBookmarkPayload {
  title: string
  url: string
  description?: string
  favicon_url?: string
  group_id?: string
}

export interface UpdateBookmarkPayload {
  title?: string
  url?: string
  description?: string
  favicon_url?: string
  group_id?: string
  sort_order?: number
}

export interface CreateAccountPayload {
  title: string
  url?: string
  username?: string
  password: string
  notes?: string
  icon_url?: string
  group_id?: string
}

export interface UpdateAccountPayload {
  title?: string
  url?: string
  username?: string
  password?: string
  notes?: string
  icon_url?: string
  group_id?: string
  favorite?: number
  sort_order?: number
}

export interface CreateNotePayload {
  title: string
  content: string
  group_id?: string
}

export interface UpdateNotePayload {
  title?: string
  content?: string
  group_id?: string
  sort_order?: number
}

export interface CreateReportPayload {
  title: string
  type: 'daily' | 'weekly' | 'monthly'
  content: string
  date: string
  week_start?: string
  week_end?: string
  month?: string
}

export interface UpdateReportPayload {
  title?: string
  content?: string
  date?: string
  week_start?: string
  week_end?: string
  month?: string
}

// App settings
export interface AppSettings {
  auto_lock_minutes: number
  theme: 'light' | 'dark' | 'system'
  language: string
}