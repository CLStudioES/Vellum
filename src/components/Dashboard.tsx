import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface EnvEntry {
  key: string;
  value: string;
  lineNumber: number;
  isComment: boolean;
  isEmpty: boolean;
  isDuplicate: boolean;
  hasFormatError: boolean;
  isSensitive: boolean;
  expandsVariables: boolean;
}

interface EnvFile {
  filename: string;
  entries: EnvEntry[];
}

interface ScanResult {
  directory: string;
  folderName: string;
  files: EnvFile[];
}

interface ProjectSummary {
  id: string;
  name: string;
  directory: string;
  ownerId: string;
  role: string;
  fileCount: number;
  entryCount: number;
  createdAt: string;
  updatedAt: string;
}

interface SessionInfo {
  userId: string;
  username: string;
}

type AssignMode = "new" | "existing";

interface DashboardProps {
  session: SessionInfo;
  onLogout: () => void;
}

export default function Dashboard({ session, onLogout }: DashboardProps) {
  const [scan, setScan] = useState<ScanResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [revealedLines, setRevealedLines] = useState<Set<string>>(new Set());
  const [collapsedFiles, setCollapsedFiles] = useState<Set<string>>(new Set());

  const [assignMode, setAssignMode] = useState<AssignMode>("new");
  const [projectName, setProjectName] = useState("");
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [savedResult, setSavedResult] = useState<ProjectSummary | null>(null);

  async function handlePickDirectory() {
    const dir = await open({ directory: true, multiple: false, title: "Select project directory" });
    if (!dir) return;

    setIsLoading(true);
    setSavedResult(null);
    try {
      const result: ScanResult = await invoke("scan_directory", { directory: dir });
      setScan(result);

      const keys = new Set<string>();
      result.files.forEach((f) =>
        f.entries.forEach((e) => {
          if (!e.isComment && !e.isEmpty && !e.hasFormatError) {
            keys.add(compositeKey(f.filename, e.key));
          }
        })
      );
      setSelected(keys);
      setRevealedLines(new Set());
      setCollapsedFiles(new Set());
    } catch (error) {
      console.error("Scan failed:", error);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleLoadProjects() {
    try {
      const result: ProjectSummary[] = await invoke("list_projects");
      setProjects(result);
    } catch (error) {
      console.error("Failed to load projects:", error);
      setProjects([]);
    }
  }

  async function handleSave() {
    if (!scan) return;
    if (assignMode === "new" && !projectName) return;
    if (assignMode === "existing" && !selectedProjectId) return;

    const filteredFiles: EnvFile[] = scan.files
      .map((f) => ({
        filename: f.filename,
        entries: f.entries.filter(
          (e) => !e.isComment && !e.isEmpty && selected.has(compositeKey(f.filename, e.key))
        ),
      }))
      .filter((f) => f.entries.length > 0);

    setIsLoading(true);
    try {
      const result: ProjectSummary = await invoke("save_to_project", {
        projectId: assignMode === "existing" ? selectedProjectId : null,
        projectName: assignMode === "new" ? projectName : "",
        directory: scan.directory,
        files: filteredFiles,
      });
      setSavedResult(result);
    } catch (error) {
      console.error("Save failed:", error);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleLogout() {
    await invoke("logout");
    onLogout();
  }

  function compositeKey(filename: string, key: string) {
    return `${filename}::${key}`;
  }

  function toggleSelected(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  }

  function toggleFileSelection(file: EnvFile) {
    const fileKeys = file.entries
      .filter((e) => !e.isComment && !e.isEmpty && !e.hasFormatError)
      .map((e) => compositeKey(file.filename, e.key));

    const allSelected = fileKeys.every((k) => selected.has(k));
    setSelected((prev) => {
      const next = new Set(prev);
      fileKeys.forEach((k) => (allSelected ? next.delete(k) : next.add(k)));
      return next;
    });
  }

  function toggleReveal(id: string) {
    setRevealedLines((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  }

  function toggleCollapse(filename: string) {
    setCollapsedFiles((prev) => {
      const next = new Set(prev);
      next.has(filename) ? next.delete(filename) : next.add(filename);
      return next;
    });
  }

  function getFileStats(file: EnvFile) {
    const data = file.entries.filter((e) => !e.isComment && !e.isEmpty);
    return {
      total: data.length,
      errors: data.filter((e) => e.hasFormatError).length,
      duplicates: data.filter((e) => e.isDuplicate).length,
      sensitive: data.filter((e) => e.isSensitive).length,
    };
  }

  const totalSelected = selected.size;

  return (
    <div className="min-h-screen bg-slate-950 text-slate-200 p-8 font-sans">
      <header className="flex justify-between items-center border-b border-slate-800 pb-6 mb-10">
        <div className="flex items-center gap-4">
          <h1 className="text-3xl font-serif text-indigo-400 font-bold">Vellum</h1>
          <span className="text-xs font-mono text-slate-600 bg-slate-900 px-2.5 py-1 rounded">
            {session.username}
          </span>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={handlePickDirectory}
            disabled={isLoading}
            className="bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white px-6 py-2 rounded-lg font-medium transition-all active:scale-95 shadow-lg shadow-indigo-500/20"
          >
            {isLoading ? "Scanning..." : "Open Directory"}
          </button>
          <button
            onClick={handleLogout}
            className="text-xs font-mono text-slate-600 hover:text-slate-400 transition-colors px-3 py-2"
          >
            Lock
          </button>
        </div>
      </header>

      <main className="max-w-3xl mx-auto">
        {savedResult && (
          <div className="mb-8 bg-emerald-950/30 border border-emerald-800/50 rounded-xl p-4 flex items-center gap-3">
            <div className="h-2 w-2 rounded-full bg-emerald-500" />
            <p className="font-mono text-sm text-emerald-400">
              Encrypted and saved:{" "}
              <span className="text-emerald-300">{savedResult.name}</span>
              <span className="text-emerald-600 ml-2">
                ({savedResult.fileCount} files, {savedResult.entryCount} entries)
              </span>
            </p>
          </div>
        )}

        {!scan && (
          <div className="border-2 border-dashed border-slate-800 rounded-xl p-16 text-center">
            <p className="text-slate-600 italic">Select a directory to begin scanning.</p>
          </div>
        )}

        {scan && (
          <>
            <div className="mb-8 bg-slate-900 border border-slate-800 rounded-xl p-5">
              <div className="flex items-center gap-3 mb-2">
                <div className="h-2.5 w-2.5 rounded-full bg-indigo-500" />
                <h2 className="font-mono text-lg text-indigo-300">{scan.folderName}</h2>
              </div>
              <p className="font-mono text-xs text-slate-600 ml-5">{scan.directory}</p>
              <div className="flex gap-4 mt-3 ml-5">
                <span className="font-mono text-xs text-slate-500">
                  {scan.files.length} env {scan.files.length === 1 ? "file" : "files"} detected
                </span>
                <span className="font-mono text-xs text-slate-500">
                  {totalSelected} variables selected
                </span>
              </div>
            </div>

            {scan.files.length === 0 && (
              <div className="border-2 border-dashed border-slate-800 rounded-xl p-12 text-center">
                <p className="text-slate-600 italic">No .env files found in this directory.</p>
              </div>
            )}

            <div className="space-y-4 mb-10">
              {scan.files.map((file) => {
                const stats = getFileStats(file);
                const isCollapsed = collapsedFiles.has(file.filename);
                const fileDataEntries = file.entries.filter((e) => !e.isComment && !e.isEmpty);
                const allFileSelected = fileDataEntries
                  .filter((e) => !e.hasFormatError)
                  .every((e) => selected.has(compositeKey(file.filename, e.key)));

                return (
                  <div
                    key={file.filename}
                    className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden"
                  >
                    <div className="px-5 py-4 flex items-center gap-3 border-b border-slate-800/50">
                      <input
                        type="checkbox"
                        checked={allFileSelected && fileDataEntries.length > 0}
                        onChange={() => toggleFileSelection(file)}
                        className="accent-indigo-500 shrink-0"
                      />
                      <button
                        onClick={() => toggleCollapse(file.filename)}
                        className="flex-1 flex items-center gap-3 text-left"
                      >
                        <span className="font-mono text-sm text-indigo-300">{file.filename}</span>
                        <div className="flex gap-2 ml-auto">
                          <StatBadge value={stats.total} label="vars" color="slate" />
                          {stats.errors > 0 && <StatBadge value={stats.errors} label="err" color="red" />}
                          {stats.duplicates > 0 && <StatBadge value={stats.duplicates} label="dup" color="amber" />}
                          {stats.sensitive > 0 && <StatBadge value={stats.sensitive} label="sensitive" color="rose" />}
                        </div>
                        <span className="text-slate-600 text-xs ml-2">{isCollapsed ? "▸" : "▾"}</span>
                      </button>
                    </div>

                    {!isCollapsed && (
                      <div className="divide-y divide-slate-800/30">
                        {file.entries.map((entry) => {
                          if (entry.isEmpty) return null;

                          if (entry.isComment) {
                            return (
                              <div key={entry.lineNumber} className="px-5 py-1.5">
                                <span className="font-mono text-xs text-slate-700">{entry.key}</span>
                              </div>
                            );
                          }

                          const cKey = compositeKey(file.filename, entry.key);
                          const isRevealed = revealedLines.has(cKey);
                          const isSelected = selected.has(cKey);

                          return (
                            <div
                              key={entry.lineNumber}
                              className={`px-5 py-2.5 flex items-center gap-3 ${
                                entry.hasFormatError
                                  ? "bg-red-950/15"
                                  : entry.isDuplicate
                                    ? "bg-amber-950/15"
                                    : ""
                              }`}
                            >
                              <input
                                type="checkbox"
                                checked={isSelected}
                                onChange={() => toggleSelected(cKey)}
                                disabled={entry.hasFormatError}
                                className="accent-indigo-500 shrink-0"
                              />

                              <span className="text-[10px] font-mono text-slate-700 w-5 text-right shrink-0">
                                {entry.lineNumber}
                              </span>

                              <span className="font-mono text-sm text-indigo-300 shrink-0">
                                {entry.key}
                              </span>

                              <span className="text-slate-700 shrink-0">=</span>

                              <span
                                className={`font-mono text-sm flex-1 min-w-0 truncate ${
                                  isRevealed
                                    ? "text-emerald-400"
                                    : "text-slate-500 select-none blur-sm"
                                }`}
                              >
                                {entry.value || (
                                  <span className="italic text-slate-700">empty</span>
                                )}
                              </span>

                              <button
                                onClick={() => toggleReveal(cKey)}
                                className="text-[10px] font-mono text-slate-600 hover:text-slate-300 transition-colors shrink-0 uppercase"
                              >
                                {isRevealed ? "Hide" : "Show"}
                              </button>

                              {entry.isSensitive && <Badge text="SENSITIVE" color="rose" />}
                              {entry.expandsVariables && <Badge text="$VAR" color="violet" />}
                              {entry.isDuplicate && <Badge text="DUP" color="amber" />}
                              {entry.hasFormatError && <Badge text="ERR" color="red" />}
                            </div>
                          );
                        })}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>

            {scan.files.length > 0 && (
              <section className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-5">
                <h3 className="text-xs font-mono uppercase tracking-[0.3em] text-slate-500">
                  Save to Vault
                </h3>

                <div className="flex gap-2">
                  <button
                    onClick={() => setAssignMode("new")}
                    className={`px-4 py-2 rounded-lg font-mono text-xs transition-colors ${
                      assignMode === "new"
                        ? "bg-indigo-600 text-white"
                        : "bg-slate-800 text-slate-400 hover:text-slate-200"
                    }`}
                  >
                    New Project
                  </button>
                  <button
                    onClick={() => {
                      setAssignMode("existing");
                      handleLoadProjects();
                    }}
                    className={`px-4 py-2 rounded-lg font-mono text-xs transition-colors ${
                      assignMode === "existing"
                        ? "bg-indigo-600 text-white"
                        : "bg-slate-800 text-slate-400 hover:text-slate-200"
                    }`}
                  >
                    Existing Project
                  </button>
                </div>

                {assignMode === "new" && (
                  <input
                    type="text"
                    placeholder="Project name"
                    value={projectName}
                    onChange={(e) => setProjectName(e.target.value)}
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 font-mono text-sm text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500"
                  />
                )}

                {assignMode === "existing" && (
                  <div className="space-y-2">
                    {projects.length === 0 && (
                      <p className="font-mono text-xs text-slate-600 text-center py-3">
                        No projects in vault yet.
                      </p>
                    )}
                    {projects.map((p) => (
                      <button
                        key={p.id}
                        onClick={() => setSelectedProjectId(p.id)}
                        className={`w-full text-left px-4 py-3 rounded-lg border font-mono text-sm transition-colors ${
                          selectedProjectId === p.id
                            ? "border-indigo-500 bg-indigo-950/30 text-indigo-300"
                            : "border-slate-700 bg-slate-800 text-slate-400 hover:border-slate-600"
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <span>{p.name}</span>
                          <div className="flex items-center gap-2">
                            <RoleBadge role={p.role} />
                            <span className="text-[10px] text-slate-600">
                              {p.fileCount} files · {p.entryCount} entries
                            </span>
                          </div>
                        </div>
                      </button>
                    ))}
                  </div>
                )}

                <div className="flex items-center justify-between pt-2">
                  <span className="font-mono text-xs text-slate-500">
                    {totalSelected} variables across {scan.files.length} files
                  </span>
                  <button
                    onClick={handleSave}
                    disabled={
                      isLoading ||
                      totalSelected === 0 ||
                      (assignMode === "new" && !projectName) ||
                      (assignMode === "existing" && !selectedProjectId)
                    }
                    className="bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white px-6 py-2 rounded-lg font-medium transition-all active:scale-95 text-sm"
                  >
                    {isLoading ? "Encrypting..." : "Encrypt & Save"}
                  </button>
                </div>
              </section>
            )}
          </>
        )}
      </main>
    </div>
  );
}

function Badge({ text, color }: { text: string; color: string }) {
  return (
    <span className={`text-[10px] bg-${color}-900/50 text-${color}-400 px-1.5 py-0.5 rounded shrink-0`}>
      {text}
    </span>
  );
}

function StatBadge({ value, label, color }: { value: number; label: string; color: string }) {
  return (
    <span className={`text-[10px] font-mono text-${color}-400 bg-${color}-900/30 px-2 py-0.5 rounded`}>
      {value} {label}
    </span>
  );
}

function RoleBadge({ role }: { role: string }) {
  const styles: Record<string, string> = {
    owner: "text-indigo-400 bg-indigo-900/30",
    editor: "text-emerald-400 bg-emerald-900/30",
    viewer: "text-slate-400 bg-slate-800",
  };
  return (
    <span className={`text-[10px] font-mono px-2 py-0.5 rounded uppercase ${styles[role] || styles.viewer}`}>
      {role}
    </span>
  );
}
