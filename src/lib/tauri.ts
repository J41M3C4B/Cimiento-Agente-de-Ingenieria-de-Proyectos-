import { invoke } from "@tauri-apps/api/core";
import { es } from "../i18n/es-MX";
import type {
  AddDocumentOutcome,
  AddNeedOutcome,
  AiCheckView,
  AiModelsView,
  ProjectJob,
  AiProvider,
  AiStatus,
  AiStatusView,
  AnswerOutcome,
  AppError,
  BackupFile,
  PinCheck,
  ScanSummary,
  BudgetItemInput,
  CreateProjectOutcome,
  Decision,
  DeleteSummary,
  ConversationView,
  DraftEditOutcome,
  DraftOutcome,
  DraftingView,
  Exported,
  ReviewView,
  DocumentSummary,
  EditOutcome,
  FileRole,
  NeedsView,
  NewProjectOutcome,
  ProfileInput,
  ProfileView,
  DonorKind,
  DraftMode,
  ProjectColor,
  ProjectRow,
  ReadingDetail,
  SaveProfileOutcome,
  Scores,
  StageName,
  SummaryEdit,
  SummaryOutcome,
  UsageReport,
  Entity,
  FieldInput,
  RosterChange,
  RosterField,
  RosterOverview,
} from "./types";

export interface AppInfo {
  name: string;
  version: string;
}

/** Errors from Rust arrive as `{ code, message }` with a friendly Spanish message. */
export function toAppError(e: unknown): AppError {
  if (e && typeof e === "object" && "message" in e && "code" in e) {
    return e as AppError;
  }
  return { code: "internal", message: es.errors.generic };
}

// Typed wrappers around invoke(). One function per Rust command.
export const appInfo = () => invoke<AppInfo>("app_info");
export const profileGet = () => invoke<ProfileView | null>("profile_get");
export const profileSave = (input: ProfileInput, decision?: Decision) =>
  invoke<SaveProfileOutcome>("profile_save", { input, decision: decision ?? null });
// The roster: staff and people served, one record each (ADR-020)
export const rosterOverview = (entity: Entity) => invoke<RosterOverview>("roster_overview", { entity });
export const rosterFieldSave = (entity: Entity, field: FieldInput) => invoke<RosterField[]>("roster_field_save", { entity, field });
export const rosterFieldDelete = (entity: Entity, key: string) => invoke<RosterField[]>("roster_field_delete", { entity, key });
export const rosterEntrySave = (entity: Entity, id: string | null, data: Record<string, string>) =>
  invoke<RosterChange>("roster_entry_save", { entity, id, data });
export const rosterEntryDelete = (entity: Entity, id: string) => invoke<RosterChange>("roster_entry_delete", { entity, id });
export const profileConfirm = () => invoke<ProfileView>("profile_confirm");
// Documents of the institution (global): the files of a call come in with their project
export const documentAddText = (displayName: string, text: string, decision?: Decision) =>
  invoke<AddDocumentOutcome>("document_add_text", {
    displayName,
    text,
    decision: decision ?? null,
  });
export const documentsList = () => invoke<DocumentSummary[]>("documents_list");
export const documentEmergencyDelete = (id: string) =>
  invoke<DeleteSummary>("document_emergency_delete", { id });
// Calls (convocatorias) belong to the project born from them: files go in as text (base64) and the
// reading happens in the background
export interface CallUpload {
  name: string;
  data: string;
  role: FileRole;
}
export const projectCreateFromCall = (
  files: CallUpload[],
  name: string,
  funder: string,
  year: number | null,
  decision?: Decision,
) =>
  invoke<NewProjectOutcome>("project_create_from_call", {
    files,
    name,
    funder: funder.trim() ? funder.trim() : null,
    year,
    decision: decision ?? null,
  });
export const callReadingGet = (id: string) => invoke<ReadingDetail>("call_reading_get", { id });
export const callReadingRetry = (id: string) => invoke<boolean>("call_reading_retry", { id });
/** The person says this is the right call. */
export const callReadingConfirm = (id: string) => invoke<boolean>("call_reading_confirm", { id });
/** Writes the «en pocas palabras» of a read call, once. */
export const callBriefMake = (id: string) => invoke<ReadingDetail>("call_brief_make", { id });
export const devLoadFixture = (name: "asilo" | "casa-hogar") =>
  invoke<ProfileView>("dev_load_fixture", { name });

// AI settings (the key goes to the Windows keychain and never comes back to the screen)
export const aiStatus = () => invoke<AiStatusView>("ai_status");
export const aiSetProvider = (provider: AiProvider) => invoke<void>("ai_set_provider", { provider });
/** The models of the active service and the thinking depth: swapped here, no rebuild needed. */
export const aiModels = () => invoke<AiModelsView>("ai_models");
export const aiSetModels = (light: string, strong: string, effort: string) =>
  invoke<void>("ai_set_models", { light, strong, effort });
export const aiSetKey =(provider: AiProvider, key: string) => invoke<void>("ai_set_key", { provider, key });
export const aiClearKey = (provider: AiProvider) => invoke<void>("ai_clear_key", { provider });
export const aiSetCap = (capMxn: number) => invoke<void>("ai_set_cap", { capMxn });
/** Checks the saved key and the models without using any of the day's answers. */
export const aiCheck = () => invoke<AiCheckView>("ai_check");
export const aiUsageReport = () => invoke<UsageReport>("ai_usage_report");

// Projects and diagnosis
/** What the AI is doing for the project right now (the person may have left the screen that started it). */
export const projectJob = (projectId: string) => invoke<ProjectJob>("project_job", { projectId });
export const projectList = () => invoke<ProjectRow[]>("project_list");
export const projectCreate = (initialRequest: string, decision?: Decision) =>
  invoke<CreateProjectOutcome>("project_create", { initialRequest, decision: decision ?? null });
export const projectDelete = (projectId: string) => invoke<boolean>("project_delete", { projectId });
export const projectSetDonorKind = (projectId: string, kind: DonorKind | null) => invoke<boolean>("project_set_donor_kind", { projectId, kind });
export const projectSetColor = (projectId: string, color: ProjectColor | null) => invoke<boolean>("project_set_color", { projectId, color });
export const projectAdvance = (projectId: string) => invoke<ProjectRow>("project_advance", { projectId });
export const projectGoBack = (projectId: string, target: StageName) =>
  invoke<ProjectRow>("project_go_back", { projectId, target });
// The guided conversation of the diagnosis (ADR-017)
export const conversationGet = (projectId: string) => invoke<ConversationView>("conversation_get", { projectId });
/** Asks for the opening question; asking again changes nothing and costs nothing. */
export const conversationStart = (projectId: string) => invoke<AnswerOutcome>("conversation_start", { projectId });
/** What the person writes; with `confirmRoot` it is the quick reply that confirms the root cause. */
export const conversationSend = (projectId: string, text: string, confirmRoot: boolean, decision?: Decision) =>
  invoke<AnswerOutcome>("conversation_send", { projectId, text, confirmRoot, decision: decision ?? null });
/** The AI owed a message and failed: asks again without the person writing anything. */
export const conversationRetry = (projectId: string) => invoke<AnswerOutcome>("conversation_retry", { projectId });
export const diagnosisSummaryGenerate = (projectId: string) =>
  invoke<SummaryOutcome>("diagnosis_summary_generate", { projectId });
export const diagnosisSummaryEdit = (projectId: string, edit: SummaryEdit, decision?: Decision) =>
  invoke<EditOutcome>("diagnosis_summary_edit", { projectId, edit, decision: decision ?? null });
export const diagnosisSummaryConfirm = (projectId: string) =>
  invoke<ConversationView>("diagnosis_summary_confirm", { projectId });

// Needs
export const needsGet = (projectId: string) => invoke<NeedsView>("needs_get", { projectId });
export const needsPropose = (projectId: string) =>
  invoke<{ view: NeedsView; ai: AiStatus }>("needs_propose", { projectId });
export const needAdd = (projectId: string, title: string, description: string, decision?: Decision) =>
  invoke<AddNeedOutcome>("need_add", { projectId, title, description, decision: decision ?? null });
export const needRate = (projectId: string, needId: string, scores: Scores) =>
  invoke<NeedsView>("need_rate", { projectId, needId, scores });
export const needSelect = (projectId: string, needId: string) =>
  invoke<NeedsView>("need_select", { projectId, needId });

// Drafting, review and the guide in Word (ADR-018)
export const draftingGet = (projectId: string) => invoke<DraftingView>("drafting_get", { projectId });
export const draftingSetAsks = (projectId: string, asks: boolean) => invoke<DraftingView>("drafting_set_asks", { projectId, asks });
export const draftingPrepare = (projectId: string) => invoke<DraftOutcome>("drafting_prepare", { projectId });
export const sectionsDraftAll = (projectId: string, mode: DraftMode) => invoke<DraftOutcome>("sections_draft_all", { projectId, mode });
export const sectionsConfirmAll = (projectId: string) => invoke<DraftingView>("sections_confirm_all", { projectId });
export const sectionDraft = (projectId: string, key: string) => invoke<DraftOutcome>("section_draft", { projectId, key });
export const sectionSave = (projectId: string, key: string, text: string, decision?: Decision) =>
  invoke<DraftEditOutcome>("section_save", { projectId, key, text, decision: decision ?? null });
export const sectionConfirm = (projectId: string, key: string) => invoke<DraftingView>("section_confirm", { projectId, key });
export const budgetSaveItem = (projectId: string, item: BudgetItemInput, decision?: Decision) =>
  invoke<DraftEditOutcome>("budget_save_item", { projectId, item, decision: decision ?? null });
export const budgetDeleteItem = (projectId: string, itemId: string) => invoke<DraftingView>("budget_delete_item", { projectId, itemId });
export const budgetConfirm = (projectId: string) => invoke<DraftingView>("budget_confirm", { projectId });
export const scheduleSaveActivity = (
  projectId: string,
  activityId: string | null,
  title: string,
  startMonth: number,
  endMonth: number,
  decision?: Decision,
) => invoke<DraftEditOutcome>("schedule_save_activity", { projectId, activityId, title, startMonth, endMonth, decision: decision ?? null });
export const scheduleDeleteActivity = (projectId: string, activityId: string) =>
  invoke<DraftingView>("schedule_delete_activity", { projectId, activityId });
export const scheduleConfirm = (projectId: string) => invoke<DraftingView>("schedule_confirm", { projectId });
export const reviewGet = (projectId: string) => invoke<ReviewView>("review_get", { projectId });
/** Writes the guide in Word in the person's Downloads folder. */
export const guideExport = (projectId: string) => invoke<Exported>("guide_export", { projectId });

// Security: PIN, encrypted backup and the scan of everything the app keeps (ADR-019)
export const pinStatus = () => invoke<boolean>("pin_status");
export const pinSet = (pin: string, current?: string) => invoke<void>("pin_set", { pin, current: current ?? null });
export const pinClear = (current: string) => invoke<void>("pin_clear", { current });
export const pinVerify = (pin: string) => invoke<PinCheck>("pin_verify", { pin });
export const securityScan = () => invoke<ScanSummary>("security_scan");
export const backupCreate = (password: string) => invoke<BackupFile>("backup_create", { password });
/** `data` is the backup file in base64. */
export const backupRestore = (data: string, password: string) => invoke<void>("backup_restore", { data, password });
