import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import AuthScreen from "./components/AuthScreen";
import Dashboard from "./components/Dashboard";

interface SessionInfo {
  userId: string;
  username: string;
}

function App() {
  const [session, setSession] = useState<SessionInfo | null>(null);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    invoke<SessionInfo | null>("get_session")
      .then((s) => setSession(s ?? null))
      .finally(() => setChecking(false));
  }, []);

  if (checking) return null;

  if (!session) {
    return <AuthScreen onAuthenticated={setSession} />;
  }

  return <Dashboard session={session} onLogout={() => setSession(null)} />;
}

export default App;
