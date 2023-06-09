import React, {
  PropsWithChildren,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { UIContext } from '../uiContext';
import { usePersistentState } from '../../hooks/usePersistentState';
import { DeviceContext } from '../deviceContext';
import { gitHubStatus } from '../../services/api';
import { SettingSections } from '../../components/Settings';

export const UIContextProvider = ({ children }: PropsWithChildren) => {
  const [isSettingsOpen, setSettingsOpen] = useState(false);
  const [isBugReportModalOpen, setBugReportModalOpen] = useState(false);
  const [settingsSection, setSettingsSection] = useState(
    SettingSections.GENERAL,
  );
  const [symbolsCollapsed, setSymbolsCollapsed] = useState(true);
  const [onBoardingState, setOnBoardingState] = usePersistentState(
    {},
    'onBoardingState',
  );
  const { isSelfServe } = useContext(DeviceContext);
  const [isGithubConnected, setGithubConnected] = useState(isSelfServe);
  const [isGithubChecked, setGithubChecked] = useState(false);

  useEffect(() => {
    if (!isSelfServe) {
      gitHubStatus().then((d) => {
        setGithubConnected(d.status === 'ok');
        setGithubChecked(true);
      });
    }
  }, []);

  const uiContextValue = useMemo(
    () => ({
      isSettingsOpen,
      setSettingsOpen,
      symbolsCollapsed,
      setSymbolsCollapsed,
      settingsSection,
      setSettingsSection,
      onBoardingState,
      setOnBoardingState,
      isBugReportModalOpen,
      setBugReportModalOpen,
      isGithubConnected,
      setGithubConnected,
      isGithubChecked,
    }),
    [
      isSettingsOpen,
      symbolsCollapsed,
      settingsSection,
      onBoardingState,
      isBugReportModalOpen,
      isGithubConnected,
      isGithubChecked,
    ],
  );
  return (
    <UIContext.Provider value={uiContextValue}>{children}</UIContext.Provider>
  );
};
