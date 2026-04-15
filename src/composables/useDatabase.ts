import { invoke } from '@tauri-apps/api/core'
import type {
  Bookmark, BookmarkGroup,
  Account, AccountGroup,
  Note, NoteGroup,
  Report,
  MediaAsset, Tag, SearchResult, SearchFilters,
  CreateBookmarkPayload, UpdateBookmarkPayload,
  CreateAccountPayload, UpdateAccountPayload,
  CreateNotePayload, UpdateNotePayload,
  CreateReportPayload, UpdateReportPayload,
} from '../types'

export function useDatabase() {
  // ===== Bookmarks =====
  async function listBookmarks(groupId: string | null = null): Promise<Bookmark[]> {
    return invoke<Bookmark[]>('list_bookmarks', { groupId })
  }

  async function getBookmark(id: string): Promise<Bookmark> {
    return invoke<Bookmark>('get_bookmark', { id })
  }

  async function createBookmark(data: CreateBookmarkPayload): Promise<Bookmark> {
    return invoke<Bookmark>('create_bookmark', { data })
  }

  async function updateBookmark(id: string, data: UpdateBookmarkPayload): Promise<Bookmark> {
    return invoke<Bookmark>('update_bookmark', { id, data })
  }

  async function deleteBookmark(id: string): Promise<void> {
    return invoke('delete_bookmark', { id })
  }

  async function listBookmarkGroups(): Promise<BookmarkGroup[]> {
    return invoke<BookmarkGroup[]>('list_bookmark_groups')
  }

  async function createBookmarkGroup(data: { name: string; icon?: string; color?: string; parent_id?: string }): Promise<BookmarkGroup> {
    return invoke<BookmarkGroup>('create_bookmark_group', { data })
  }

  async function updateBookmarkGroup(id: string, data: Partial<BookmarkGroup>): Promise<BookmarkGroup> {
    return invoke<BookmarkGroup>('update_bookmark_group', { id, data })
  }

  async function deleteBookmarkGroup(id: string): Promise<void> {
    return invoke('delete_bookmark_group', { id })
  }

  // ===== Accounts =====
  async function listAccounts(groupId: string | null = null): Promise<Account[]> {
    return invoke<Account[]>('list_accounts', { groupId })
  }

  async function getAccount(id: string): Promise<Account> {
    return invoke<Account>('get_account', { id })
  }

  async function createAccount(data: CreateAccountPayload): Promise<Account> {
    return invoke<Account>('create_account', { data })
  }

  async function updateAccount(id: string, data: UpdateAccountPayload): Promise<Account> {
    return invoke<Account>('update_account', { id, data })
  }

  async function deleteAccount(id: string): Promise<void> {
    return invoke('delete_account', { id })
  }

  async function generatePassword(length: number = 16, options?: any): Promise<string> {
    return invoke<string>('generate_password', { length, options })
  }

  async function listAccountGroups(): Promise<AccountGroup[]> {
    return invoke<AccountGroup[]>('list_account_groups')
  }

  // ===== Notes =====
  async function listNotes(groupId: string | null = null): Promise<Note[]> {
    return invoke<Note[]>('list_notes', { groupId })
  }

  async function getNote(id: string): Promise<Note> {
    return invoke<Note>('get_note', { id })
  }

  async function createNote(data: CreateNotePayload): Promise<Note> {
    return invoke<Note>('create_note', { data })
  }

  async function updateNote(id: string, data: UpdateNotePayload): Promise<Note> {
    return invoke<Note>('update_note', { id, data })
  }

  async function deleteNote(id: string): Promise<void> {
    return invoke('delete_note', { id })
  }

  async function encryptNote(id: string, secondaryPassword: string): Promise<void> {
    return invoke('encrypt_note', { id, secondaryPassword })
  }

  async function decryptNote(id: string, secondaryPassword: string): Promise<Note> {
    return invoke<Note>('decrypt_note', { id, secondaryPassword })
  }

  // ===== Reports =====
  async function listReports(type: string = ''): Promise<Report[]> {
    return invoke<Report[]>('list_reports', { type })
  }

  async function getReport(id: string): Promise<Report> {
    return invoke<Report>('get_report', { id })
  }

  async function createReport(data: CreateReportPayload): Promise<Report> {
    return invoke<Report>('create_report', { data })
  }

  async function updateReport(id: string, data: UpdateReportPayload): Promise<Report> {
    return invoke<Report>('update_report', { id, data })
  }

  async function deleteReport(id: string): Promise<void> {
    return invoke('delete_report', { id })
  }

  // ===== Media =====
  async function listMediaAssets(): Promise<MediaAsset[]> {
    return invoke<MediaAsset[]>('list_media_assets')
  }

  async function uploadMedia(payload: { file_path: string; storage_type: string; description?: string }): Promise<MediaAsset> {
    return invoke<MediaAsset>('upload_media', { payload })
  }

  async function deleteMediaAsset(id: string): Promise<void> {
    return invoke('delete_media_asset', { id })
  }

  // ===== Tags =====
  async function listTags(): Promise<Tag[]> {
    return invoke<Tag[]>('list_tags')
  }

  async function createTag(data: { name: string; color?: string; icon?: string }): Promise<Tag> {
    return invoke<Tag>('create_tag', { data })
  }

  async function updateTag(id: string, data: { name?: string; color?: string; icon?: string }): Promise<Tag> {
    return invoke<Tag>('update_tag', { id, data })
  }

  async function deleteTag(id: string): Promise<void> {
    return invoke('delete_tag', { id })
  }

  async function tagEntity(tagId: string, entityType: string, entityId: string): Promise<void> {
    return invoke('tag_entity', { tagId, entityType, entityId })
  }

  async function untagEntity(tagId: string, entityType: string, entityId: string): Promise<void> {
    return invoke('untag_entity', { tagId, entityType, entityId })
  }

  // ===== Search =====
  async function search(query: string, filters?: SearchFilters): Promise<SearchResult[]> {
    return invoke<SearchResult[]>('search', { query, filters })
  }

  return {
    // Bookmarks
    listBookmarks, getBookmark, createBookmark, updateBookmark, deleteBookmark,
    listBookmarkGroups, createBookmarkGroup, updateBookmarkGroup, deleteBookmarkGroup,
    // Accounts
    listAccounts, getAccount, createAccount, updateAccount, deleteAccount,
    listAccountGroups, generatePassword,
    // Notes
    listNotes, getNote, createNote, updateNote, deleteNote,
    encryptNote, decryptNote,
    // Reports
    listReports, getReport, createReport, updateReport, deleteReport,
    // Media
    listMediaAssets, uploadMedia, deleteMediaAsset,
    // Tags
    listTags, createTag, updateTag, deleteTag, tagEntity, untagEntity,
    // Search
    search,
  }
}