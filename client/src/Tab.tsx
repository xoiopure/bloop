import React, { useMemo, useState } from 'react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { SearchContext } from './context/searchContext';
import Settings from './components/Settings';
import { FilterType, RepoType, UITabType } from './types/general';
import { DeviceContextType } from './context/deviceContext';
import { RepositoriesContext } from './context/repositoriesContext';
import './index.css';
import { AnalyticsContextProvider } from './context/providers/AnalyticsContextProvider';
import ReportBugModal from './components/ReportBugModal';
import { UIContextProvider } from './context/providers/UiContextProvider';
import { DeviceContextProvider } from './context/providers/DeviceContextProvider';
import { AppNavigationProvider } from './hooks/useAppNavigation';
import SearchPage from './pages/Search';

type Props = {
  deviceContextValue: DeviceContextType;
  isActive: boolean;
  tab: UITabType;
};

function Tab({ deviceContextValue, isActive, tab }: Props) {
  const [inputValue, setInputValue] = useState('');
  const [filters, setFilters] = useState<FilterType[]>([]);
  const [repositories, setRepositories] = useState<RepoType[]>([]);
  const [searchHistory, setSearchHistory] = useState<string[]>(
    tab.searchHistory || [],
  );
  const [lastQueryTime, setLastQueryTime] = useState(3);
  const [globalRegex, setGlobalRegex] = useState(false);

  const searchContextValue = useMemo(
    () => ({
      inputValue,
      setInputValue,
      searchHistory,
      setSearchHistory,
      filters,
      setFilters,
      lastQueryTime,
      setLastQueryTime,
      globalRegex,
      setGlobalRegex,
    }),
    [inputValue, filters, searchHistory, lastQueryTime, globalRegex],
  );

  const reposContextValue = useMemo(
    () => ({
      repositories,
      setRepositories,
      localSyncError: false,
      githubSyncError: false,
    }),
    [repositories],
  );

  return (
    <div className={`${isActive ? '' : 'hidden'} `}>
      <BrowserRouter>
        <AnalyticsContextProvider deviceId={deviceContextValue.deviceId}>
          <DeviceContextProvider deviceContextValue={deviceContextValue}>
            <UIContextProvider>
              <SearchContext.Provider value={searchContextValue}>
                <RepositoriesContext.Provider value={reposContextValue}>
                  <AppNavigationProvider>
                    <Routes>
                      <Route path="*" element={<SearchPage />} />
                    </Routes>
                    <Settings />
                    <ReportBugModal />
                  </AppNavigationProvider>
                </RepositoriesContext.Provider>
              </SearchContext.Provider>
            </UIContextProvider>
          </DeviceContextProvider>
        </AnalyticsContextProvider>
      </BrowserRouter>
    </div>
  );
}

export default Tab;
