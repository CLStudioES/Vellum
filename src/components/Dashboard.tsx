import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  SessionInfo, ProjectResponse, SaveResult, EnvFile,
  ScanResult, EntryPayload, RemoteEntry,
} from "../types";

type View = "home" | "scan" | "project" | "profile";
type AssignMode = "new" | "existing";

interface DashboardProps {
  session: SessionInfo;
  onLogout: () => void;
}

export default function Dashboard({ session, onLogout }: DashboardProps) {
  const [view, setView] = useState<View>("home");
  const [projects, setProjects] = useState<ProjectResponse[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [profileOpen, setProfileOpen] = useState(false);
  const profileRef = useRef<HTMLDivElement>(null);

  // Close dropdown on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (profileRef.current && !profileRef.current.contains(e.target as Node)) {
        setProfileOpen(false);
      }
    }
    if (profileOpen) document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [profileOpen]);

  // Project detail state
  const [activeProject, setActiveProject] = useState<ProjectResponse | null>(null);
  const [projectEntries, setProjectEntries] = useState<RemoteEntry[]>([]);
  const [revealedProjectKeys, setRevealedProjectKeys] = useState<Set<string>>(new Set());
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  // Scan state
  const [scan, setScan] = useState<ScanResult | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [revealedLines, setRevealedLines] = useState<Set<string>>(new Set());
  const [collapsedFiles, setCollapsedFiles] = useState<Set<string>>(new Set());
  const [assignMode, setAssignMode] = useState<AssignMode>("new");
  const [projectName, setProjectName] = useState("");
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);

  useEffect(() => { loadProjects(); }, []);

  async function loadProjects() {
    try {
      const result: ProjectResponse[] = await invoke("list_projects");
      setProjects(result);
    } catch (error) {
      console.error("Failed to load projects:", error);
    }
  }

  async function handleOpenProject(project: ProjectResponse) {
    setIsLoading(true);
    try {
      const entries: RemoteEntry[] = await invoke("get_project_entries", { projectId: project.id });
      setActiveProject(project);
      setProjectEntries(entries);
      setRevealedProjectKeys(new Set());
      setView("project");
    } catch (error) {
      console.error("Failed to load project:", error);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleCopyValue(value: string, rKey: string) {
    await navigator.clipboard.writeText(value);
    setCopiedKey(rKey);
    setTimeout(() => setCopiedKey(null), 1500);
  }

  async function handlePickDirectory() {
    const dir = await open({ directory: true, multiple: false, title: "Select project directory" });
    if (!dir) return;

    setIsLoading(true);
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
      setAssignMode("new");
      setProjectName("");
      setSelectedProjectId(null);
      setView("scan");
    } catch (error) {
      console.error("Scan failed:", error);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleSave() {
    if (!scan) return;
    if (assignMode === "new" && !projectName) return;
    if (assignMode === "existing" && !selectedProjectId) return;

    const entries: EntryPayload[] = [];
    scan.files.forEach((f) => {
      f.entries.forEach((e) => {
        if (e.isComment || e.isEmpty) return;
        if (!selected.has(compositeKey(f.filename, e.key))) return;
        entries.push({
          envFile: f.filename,
          key: e.key,
          encryptedValue: e.value,
          isSensitive: e.isSensitive,
        });
      });
    });

    setIsLoading(true);
    try {
      const result: SaveResult = await invoke("save_to_project", {
        projectId: assignMode === "existing" ? selectedProjectId : null,
        projectName: assignMode === "new" ? projectName : "",
        entries,
      });

      if (result.newCount === 0 && result.skippedCount > 0) {
        setToast(`No new entries — ${result.skippedCount} already exist in "${result.projectName}"`);
      } else if (result.skippedCount > 0) {
        setToast(`Saved ${result.newCount} new entries to "${result.projectName}" (${result.skippedCount} duplicates skipped)`);
      } else {
        setToast(`Saved ${result.newCount} entries to "${result.projectName}"`);
      }

      setScan(null);
      setView("home");
      loadProjects();
    } catch (error) {
      console.error("Save failed:", error);
    } finally {
      setIsLoading(false);
    }
  }

  function handleBackToHome() {
    setScan(null);
    setActiveProject(null);
    setProjectEntries([]);
    setView("home");
    loadProjects();
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
        <button onClick={handleBackToHome} className="text-3xl font-serif text-indigo-400 font-bold hover:text-indigo-300 transition-colors cursor-pointer">
          Vellum
        </button>
        <div className="flex items-center gap-3">
          <button
            onClick={handlePickDirectory}
            disabled={isLoading}
            className="h-9 w-9 flex items-center justify-center bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium transition-all active:scale-95 shadow-lg shadow-indigo-500/20 text-lg leading-none cursor-pointer"
            title="Scan directory"
          >
            +
          </button>
          <div className="relative" ref={profileRef}>
            <button
              onClick={() => setProfileOpen(!profileOpen)}
              className="h-9 flex items-center gap-2 bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-lg px-3 transition-colors cursor-pointer"
            >
              <div className="h-5 w-5 rounded-full bg-indigo-600 flex items-center justify-center">
                <span className="text-[10px] font-mono text-white uppercase">{session.username[0]}</span>
              </div>
              <span className="text-xs font-mono text-slate-400">{session.username}</span>
              <span className="text-slate-600 text-[10px]">▾</span>
            </button>
            {profileOpen && (
              <div className="absolute right-0 top-full mt-2 w-48 bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-xl shadow-black/30 z-50">
                <button
                  onClick={() => { setProfileOpen(false); setView("profile"); }}
                  className="w-full text-left px-4 py-2.5 text-xs font-mono text-slate-400 hover:bg-slate-800 transition-colors cursor-pointer"
                >
                  Profile
                </button>
                <button
                  onClick={() => { setProfileOpen(false); handleLogout(); }}
                  className="w-full text-left px-4 py-2.5 text-xs font-mono text-rose-400 hover:bg-rose-950/20 transition-colors cursor-pointer"
                >
                  Sign Out
                </button>
              </div>
            )}
          </div>
        </div>
      </header>

      <main className="max-w-3xl mx-auto">
        {/* Toast */}
        {toast && (
          <div className="mb-8 bg-emerald-950/30 border border-emerald-800/50 rounded-xl p-4 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="h-2 w-2 rounded-full bg-emerald-500" />
              <p className="font-mono text-sm text-emerald-400">{toast}</p>
            </div>
            <button onClick={() => setToast(null)} className="text-emerald-600 hover:text-emerald-400 text-xs font-mono cursor-pointer">
              Dismiss
            </button>
          </div>
        )}

        {/* HOME VIEW */}
        {view === "home" && (
          <>
            <div className="flex items-center gap-2 mb-6">
              <div className="h-2 w-2 rounded-full bg-indigo-500 animate-pulse" />
              <h2 className="text-xs font-mono uppercase tracking-[0.3em] text-slate-500">
                Your Projects
              </h2>
            </div>

            {projects.length === 0 ? (
              <div className="border-2 border-dashed border-slate-800 rounded-xl p-16 text-center">
                <p className="text-slate-600 italic mb-4">No projects yet.</p>
                <p className="text-slate-700 text-xs font-mono">Scan a directory to create your first project.</p>
              </div>
            ) : (
              <div className="space-y-3">
                {projects.map((p) => (
                  <button
                    key={p.id}
                    onClick={() => handleOpenProject(p)}
                    className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex items-center justify-between hover:border-indigo-500/50 transition-colors cursor-pointer w-full text-left"
                  >
                    <div>
                      <h3 className="font-mono text-sm text-indigo-300">{p.name}</h3>
                      <p className="text-[10px] font-mono text-slate-600 mt-1">
                        Updated {new Date(p.updatedAt).toLocaleDateString()}
                      </p>
                    </div>
                    <RoleBadge role={p.role} />
                  </button>
                ))}
              </div>
            )}
          </>
        )}

        {/* SCAN VIEW */}
        {view === "scan" && scan && (
          <>
            <button
              onClick={handleBackToHome}
              className="text-xs font-mono text-slate-600 hover:text-slate-400 transition-colors mb-6 flex items-center gap-1 cursor-pointer"
            >
              ← Back to projects
            </button>

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
                  <div key={file.filename} className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
                    <div className="px-5 py-4 flex items-center gap-3 border-b border-slate-800/50">
                      <input
                        type="checkbox"
                        checked={allFileSelected && fileDataEntries.length > 0}
                        onChange={() => toggleFileSelection(file)}
                        className="accent-indigo-500 shrink-0"
                      />
                      <button
                        onClick={() => toggleCollapse(file.filename)}
                        className="flex-1 flex items-center gap-3 text-left cursor-pointer"
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
                                entry.hasFormatError ? "bg-red-950/15" : entry.isDuplicate ? "bg-amber-950/15" : ""
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
                              <span className="font-mono text-sm text-indigo-300 shrink-0">{entry.key}</span>
                              <span className="text-slate-700 shrink-0">=</span>
                              <span
                                className={`font-mono text-sm flex-1 min-w-0 truncate ${
                                  isRevealed ? "text-emerald-400" : "text-slate-500 select-none blur-sm"
                                }`}
                              >
                                {entry.value || <span className="italic text-slate-700">empty</span>}
                              </span>
                              <button
                                onClick={() => toggleReveal(cKey)}
                                className="text-[10px] font-mono text-slate-600 hover:text-slate-300 transition-colors shrink-0 uppercase cursor-pointer"
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

            {/* Save panel */}
            {scan.files.length > 0 && (
              <section className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-5">
                <h3 className="text-xs font-mono uppercase tracking-[0.3em] text-slate-500">
                  Save to Vault
                </h3>

                <div className="flex gap-2">
                  <button
                    onClick={() => setAssignMode("new")}
                    className={`px-4 py-2 rounded-lg font-mono text-xs transition-colors cursor-pointer ${
                      assignMode === "new" ? "bg-indigo-600 text-white" : "bg-slate-800 text-slate-400 hover:text-slate-200"
                    }`}
                  >
                    New Project
                  </button>
                  <button
                    onClick={() => { setAssignMode("existing"); loadProjects(); }}
                    className={`px-4 py-2 rounded-lg font-mono text-xs transition-colors cursor-pointer ${
                      assignMode === "existing" ? "bg-indigo-600 text-white" : "bg-slate-800 text-slate-400 hover:text-slate-200"
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
                      <p className="font-mono text-xs text-slate-600 text-center py-3">No projects yet.</p>
                    )}
                    {projects.map((p) => (
                      <button
                        key={p.id}
                        onClick={() => setSelectedProjectId(p.id)}
                        className={`w-full text-left px-4 py-3 rounded-lg border font-mono text-sm transition-colors cursor-pointer ${
                          selectedProjectId === p.id
                            ? "border-indigo-500 bg-indigo-950/30 text-indigo-300"
                            : "border-slate-700 bg-slate-800 text-slate-400 hover:border-slate-600"
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <span>{p.name}</span>
                          <RoleBadge role={p.role} />
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
                    className="bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white px-6 py-2 rounded-lg font-medium transition-all active:scale-95 text-sm cursor-pointer"
                  >
                    {isLoading ? "Saving..." : "Save to Vault"}
                  </button>
                </div>
              </section>
            )}
          </>
        )}

        {/* PROJECT DETAIL VIEW */}
        {view === "project" && activeProject && (
          <>
            <button
              onClick={handleBackToHome}
              className="text-xs font-mono text-slate-600 hover:text-slate-400 transition-colors mb-6 flex items-center gap-1 cursor-pointer"
            >
              ← Back to projects
            </button>

            <div className="mb-8 bg-slate-900 border border-slate-800 rounded-xl p-5">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-3">
                  <div className="h-2.5 w-2.5 rounded-full bg-indigo-500" />
                  <h2 className="font-mono text-lg text-indigo-300">{activeProject.name}</h2>
                </div>
                <RoleBadge role={activeProject.role} />
              </div>
              <p className="font-mono text-xs text-slate-600 ml-5">
                {projectEntries.length} entries · Updated {new Date(activeProject.updatedAt).toLocaleDateString()}
              </p>
            </div>

            {projectEntries.length === 0 ? (
              <div className="border-2 border-dashed border-slate-800 rounded-xl p-12 text-center">
                <p className="text-slate-600 italic">No entries in this project yet.</p>
              </div>
            ) : (
              <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden divide-y divide-slate-800/30">
                {projectEntries.map((entry) => {
                  const rKey = `${entry.envFile}::${entry.key}`;
                  const isRevealed = revealedProjectKeys.has(rKey);

                  return (
                    <div key={entry.id} className="px-5 py-2.5 flex items-center gap-3">
                      <span className="font-mono text-xs text-slate-600 shrink-0 min-w-[120px] truncate">
                        {entry.envFile}
                      </span>
                      <span className={`font-mono text-sm shrink-0 ${entry.isSensitive ? "text-rose-400" : "text-indigo-300"}`}>
                        {entry.key}
                      </span>
                      <span className="text-slate-700 shrink-0">=</span>
                      <span
                        className={`font-mono text-sm flex-1 min-w-0 truncate ${
                          isRevealed ? "text-emerald-400" : "text-slate-500 select-none blur-sm"
                        }`}
                      >
                        {entry.encryptedValue}
                      </span>
                      <button
                        onClick={() => {
                          setRevealedProjectKeys((prev) => {
                            const next = new Set(prev);
                            next.has(rKey) ? next.delete(rKey) : next.add(rKey);
                            return next;
                          });
                        }}
                        className="text-[10px] font-mono text-slate-600 hover:text-slate-300 transition-colors shrink-0 uppercase cursor-pointer"
                      >
                        {isRevealed ? "Hide" : "Show"}
                      </button>
                      <button
                        onClick={() => handleCopyValue(entry.encryptedValue, rKey)}
                        className="text-[10px] font-mono text-slate-600 hover:text-slate-300 transition-colors shrink-0 uppercase cursor-pointer"
                      >
                        {copiedKey === rKey ? "Done" : "Copy"}
                      </button>
                    </div>
                  );
                })}
              </div>
            )}
          </>
        )}

        {/* PROFILE VIEW */}
        {view === "profile" && (
          <>
            <button
              onClick={handleBackToHome}
              className="text-xs font-mono text-slate-600 hover:text-slate-400 transition-colors mb-6 flex items-center gap-1 cursor-pointer"
            >
              ← Back to projects
            </button>

            <div className="flex items-center gap-2 mb-6">
              <div className="h-2 w-2 rounded-full bg-indigo-500" />
              <h2 className="text-xs font-mono uppercase tracking-[0.3em] text-slate-500">Profile</h2>
            </div>

            <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
              <div className="p-6 flex items-center gap-4 border-b border-slate-800/50">
                <div className="h-12 w-12 rounded-full bg-indigo-600 flex items-center justify-center">
                  <span className="text-lg font-mono text-white uppercase">{session.username[0]}</span>
                </div>
                <div>
                  <p className="font-mono text-sm text-slate-200">{session.username}</p>
                  <p className="font-mono text-[10px] text-slate-600 mt-1">Signed in</p>
                </div>
              </div>

              <div className="divide-y divide-slate-800/30">
                <div className="px-6 py-4 flex items-center justify-between">
                  <span className="text-xs font-mono text-slate-500">Username</span>
                  <span className="font-mono text-xs text-slate-400">{session.username}</span>
                </div>
                <div className="px-6 py-4 flex items-center justify-between">
                  <span className="text-xs font-mono text-slate-500">Projects</span>
                  <span className="font-mono text-xs text-slate-400">{projects.length}</span>
                </div>
                <div className="px-6 py-4 flex items-center justify-between">
                  <span className="text-xs font-mono text-slate-500">Session</span>
                  <span className="font-mono text-[10px] text-emerald-600">Active</span>
                </div>
              </div>

              <div className="p-6 border-t border-slate-800/50">
                <button
                  onClick={handleLogout}
                  className="w-full bg-rose-950/30 border border-rose-900/50 text-rose-400 py-2.5 rounded-lg font-mono text-xs hover:bg-rose-950/50 transition-colors cursor-pointer"
                >
                  Sign Out
                </button>
              </div>
            </div>
          </>
        )}
      </main>
    </div>
  );
}

function Badge({ text, color }: { text: string; color: "rose" | "violet" | "amber" | "red" }) {
  const styles = {
    rose: "bg-rose-900/50 text-rose-400",
    violet: "bg-violet-900/50 text-violet-400",
    amber: "bg-amber-900/50 text-amber-400",
    red: "bg-red-900/50 text-red-400",
  };
  return (
    <span className={`text-[10px] px-1.5 py-0.5 rounded shrink-0 ${styles[color]}`}>
      {text}
    </span>
  );
}

function StatBadge({ value, label, color }: { value: number; label: string; color: "slate" | "red" | "amber" | "rose" }) {
  const styles = {
    slate: "text-slate-400 bg-slate-900/30",
    red: "text-red-400 bg-red-900/30",
    amber: "text-amber-400 bg-amber-900/30",
    rose: "text-rose-400 bg-rose-900/30",
  };
  return (
    <span className={`text-[10px] font-mono px-2 py-0.5 rounded ${styles[color]}`}>
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
