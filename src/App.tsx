import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ThemeProvider } from './context/ThemeContext';
import { AppProvider } from './context/AppContext';
import AppShell from './components/layout/AppShell';
import Dashboard from './routes/Dashboard';
import Credentials from './routes/Credentials';
import Domains from './routes/Domains';
import DomainDetail from './routes/DomainDetail';
import Search from './routes/Search';
import Settings from './routes/Settings';

export default function App() {
  return (
    <ThemeProvider>
      <AppProvider>
        <BrowserRouter>
          <Routes>
            <Route element={<AppShell />}>
              <Route path="/" element={<Dashboard />} />
              <Route path="/credentials" element={<Credentials />} />
              <Route path="/domains" element={<Domains />} />
              <Route path="/domains/:provider/:domainId" element={<DomainDetail />} />
              <Route path="/search" element={<Search />} />
              <Route path="/settings" element={<Settings />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </AppProvider>
    </ThemeProvider>
  );
}
