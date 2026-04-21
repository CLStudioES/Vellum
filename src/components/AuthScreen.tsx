import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SessionInfo } from "../types";

interface AuthScreenProps {
  onAuthenticated: (session: SessionInfo) => void;
}

type Mode = "login" | "register";

export default function AuthScreen({ onAuthenticated }: AuthScreenProps) {
  const [mode, setMode] = useState<Mode>("login");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);

    if (mode === "register" && password !== confirmPassword) {
      setError("Passwords don't match");
      return;
    }

    setIsLoading(true);
    try {
      const session: SessionInfo = await invoke(mode, { username, password });
      onAuthenticated(session);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  function switchMode() {
    setMode(mode === "login" ? "register" : "login");
    setError(null);
    setConfirmPassword("");
  }

  return (
    <div className="min-h-screen bg-slate-950 flex items-center justify-center p-8">
      <div className="w-full max-w-sm">
        <h1 className="text-3xl font-serif text-indigo-400 font-bold text-center mb-2">Vellum</h1>
        <p className="text-xs font-mono text-slate-600 text-center mb-10 uppercase tracking-[0.3em]">
          Sovereign Vault
        </p>

        <form onSubmit={handleSubmit} className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-4">
          <h2 className="text-xs font-mono uppercase tracking-[0.3em] text-slate-500">
            {mode === "login" ? "Sign In" : "Create Account"}
          </h2>

          <input
            type="text"
            placeholder="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            autoFocus
            className="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-2.5 font-mono text-sm text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500"
          />

          <input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-2.5 font-mono text-sm text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500"
          />

          {mode === "register" && (
            <input
              type="password"
              placeholder="Confirm password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-2.5 font-mono text-sm text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500"
            />
          )}

          {error && (
            <p className="font-mono text-xs text-rose-400 bg-rose-950/30 border border-rose-900/50 rounded-lg px-3 py-2">
              {error}
            </p>
          )}

          <button
            type="submit"
            disabled={isLoading || !username || !password}
            className="w-full bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white py-2.5 rounded-lg font-medium transition-all active:scale-[0.98] text-sm"
          >
            {isLoading ? "Connecting..." : mode === "login" ? "Sign In" : "Create Account"}
          </button>

          <button
            type="button"
            onClick={switchMode}
            className="w-full text-xs font-mono text-slate-600 hover:text-slate-400 transition-colors py-1"
          >
            {mode === "login" ? "No account yet? Create one" : "Already have an account? Sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
