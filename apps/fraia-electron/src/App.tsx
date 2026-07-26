import { useEffect, useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AppShell } from './components/layout/AppShell';
import { loadDefaultProject } from './lib/defaultProject';
import { useThemeMode } from './lib/theme';
import type { WorkbenchState } from './lib/types';

export default function App() {
  const [state, setState] = useState<WorkbenchState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { themeMode, setThemeMode } = useThemeMode();

  useEffect(() => {
    let cancelled = false;
    loadDefaultProject()
      .then((loaded) => {
        if (!cancelled) setState(loaded);
      })
      .catch((caught) => {
        if (!cancelled) setError(caught.message);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (error && !state) {
    return (
      <main className="flex h-screen items-center justify-center p-6">
        <Alert variant="destructive" className="max-w-xl">
          <AlertTitle>Fraia backend unavailable</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </main>
    );
  }

  return <AppShell state={state} onState={setState} themeMode={themeMode} onThemeModeChange={setThemeMode} />;
}
