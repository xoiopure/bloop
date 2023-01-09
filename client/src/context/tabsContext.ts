import { createContext } from 'react';
import { UITabType } from '../types/general';

type ContextType = {
  tabs: UITabType[];
  activeTab: number;
  handleAddTab: (t: UITabType) => void;
  setActiveTab: (t: number) => void;
};

export const TabsContext = createContext<ContextType>({
  tabs: [{ key: 0 }],
  activeTab: 0,
  handleAddTab: () => {},
  setActiveTab: () => {},
});
