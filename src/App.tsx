import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [files, setFiles] = useState<string[]>([]);
  const [isScanning, setIsScanning] = useState(false);

  async function handleScan() {
    setIsScanning(true);
    try {
      // Calls the Rust function scan_env_files from src-tauri/src/lib.rs
      const result: string[] = await invoke("scan_env_files", { directory: "." });
      setFiles(result);
    } catch (error) {
      console.error("Scan failed:", error);
    } finally {
      setIsScanning(false);
    }
  }

  return (
    <div className="min-h-screen bg-slate-950 text-slate-200 p-8 font-sans">
      <header className="flex justify-between items-center border-b border-slate-800 pb-6 mb-10">
        <h1 className="text-3xl font-serif text-indigo-400 font-bold">Vellum</h1>
        <button 
          onClick={handleScan}
          disabled={isScanning}
          className="bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white px-6 py-2 rounded-lg font-medium transition-all active:scale-95 shadow-lg shadow-indigo-500/20"
        >
          {isScanning ? "Scanning..." : "Scan Directory"}
        </button>
      </header>

      <main className="max-w-2xl mx-auto">
        <div className="flex items-center gap-2 mb-4">
          <div className="h-2 w-2 rounded-full bg-indigo-500 animate-pulse"></div>
          <h2 className="text-xs font-mono uppercase tracking-[0.3em] text-slate-500">Detected Environments</h2>
        </div>

        <div className="grid gap-3">
          {files.length > 0 ? (
            files.map((file) => (
              <div key={file} className="bg-slate-900 border border-slate-800 p-4 rounded-xl flex justify-between items-center group hover:border-indigo-500/50 transition-colors">
                <span className="font-mono text-indigo-300">{file}</span>
                <span className="text-[10px] bg-slate-800 text-slate-400 px-2 py-1 rounded uppercase tracking-wider group-hover:text-indigo-400 transition-colors">Local</span>
              </div>
            ))
          ) : (
            <div className="border-2 border-dashed border-slate-800 rounded-xl p-12 text-center">
              <p className="text-slate-600 italic">No .env files detected. Click scan to begin.</p>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

export default App;