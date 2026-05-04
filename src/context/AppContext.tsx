import { createContext, useContext, useReducer, type ReactNode } from 'react';
import type { ProviderCredential } from '../types';

interface AppState {
  credentials: ProviderCredential[];
  loaded: boolean;
}

type AppAction =
  | { type: 'SET_CREDENTIALS'; payload: ProviderCredential[] }
  | { type: 'ADD_CREDENTIAL'; payload: ProviderCredential }
  | { type: 'REMOVE_CREDENTIAL'; payload: string };

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'SET_CREDENTIALS':
      return { ...state, credentials: action.payload, loaded: true };
    case 'ADD_CREDENTIAL':
      return {
        ...state,
        credentials: [...state.credentials, action.payload],
      };
    case 'REMOVE_CREDENTIAL':
      return {
        ...state,
        credentials: state.credentials.filter((c) => c.id !== action.payload),
      };
    default:
      return state;
  }
}

interface AppContextType {
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, {
    credentials: [],
    loaded: false,
  });

  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}
