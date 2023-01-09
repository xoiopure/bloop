import React, { useCallback, useMemo, useState } from 'react';
import { DeviceContextType } from './context/deviceContext';
import './index.css';
import Tab from './Tab';
import { TabsContext } from './context/tabsContext';
import { UITabType } from './types/general';
import { getJsonFromStorage, SEARCH_HISTORY_KEY } from './services/storage';

type Props = {
  deviceContextValue: DeviceContextType;
};

function App({ deviceContextValue }: Props) {
  const [tabs, setTabs] = useState<UITabType[]>([
    { key: 0, searchHistory: getJsonFromStorage(SEARCH_HISTORY_KEY) || [] },
  ]);
  const [activeTab, setActiveTab] = useState(0);

  const handleAddTab = useCallback((newTab: UITabType) => {
    setTabs((prev) => [...prev, newTab]);
    setActiveTab(newTab.key);
  }, []);

  const contextValue = useMemo(
    () => ({
      tabs,
      activeTab,
      handleAddTab,
      setActiveTab,
    }),
    [tabs, activeTab, handleAddTab],
  );

  return (
    <TabsContext.Provider value={contextValue}>
      {tabs.map((t, i) => (
        <Tab
          key={t.key}
          deviceContextValue={deviceContextValue}
          isActive={i === activeTab}
          tab={t}
        />
      ))}
    </TabsContext.Provider>
  );
}

export default App;
