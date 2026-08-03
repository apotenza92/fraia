import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from '@/components/ui/menubar';
import { Separator } from '@/components/ui/separator';
import { AiProvidersDialog } from '@/components/ai/AiProvidersDialog';
import { UpdateDialog } from '@/components/updates/UpdateDialog';
import type { UpdateFrequency, UpdateStatus } from '@/lib/updateStatus';
import { Fragment, useEffect, useState } from 'react';
import { CHROME } from './chromeMetrics';

type MenuItem = {
  label: string;
  onSelect?: () => void;
  disabled?: boolean;
  detail?: React.ReactNode;
};

export function AppMenuBar() {
  const [fraiaAiOpen, setFraiaAiOpen] = useState(false);
  const [updateOpen, setUpdateOpen] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const [updateAction, setUpdateAction] = useState<'checking' | 'installing' | null>(null);
  const [productName, setProductName] = useState('Fraia');

  useEffect(() => {
    let active = true;
    void window.fraia.applicationMetadata?.().then((metadata) => {
      if (active && metadata?.productName) {
        setProductName(metadata.productName);
      }
    });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    let active = true;
    void window.fraia.updateStatus?.()
      .then((status) => {
        if (active) setUpdateStatus(status);
      })
      .catch(() => {});
    const unsubscribeStatus = window.fraia.onUpdateStatus?.((status) => {
      if (!active) return;
      setUpdateStatus(status);
      if (status.phase === 'ready') setUpdateOpen(true);
    });
    const unsubscribeOpen = window.fraia.onOpenUpdateDialog?.(() => {
      if (active) setUpdateOpen(true);
    });
    return () => {
      active = false;
      unsubscribeStatus?.();
      unsubscribeOpen?.();
    };
  }, []);

  useEffect(() => {
    document.title = productName;
  }, [productName]);
  const reloadWindow = () => {
    if (window.fraia.reloadWindow) {
      void window.fraia.reloadWindow();
      return;
    }
    window.location.reload();
  };
  const forceReloadWindow = () => {
    if (window.fraia.forceReloadWindow) {
      void window.fraia.forceReloadWindow();
      return;
    }
    window.location.reload();
  };
  const quitApp = () => {
    if (window.fraia.quitApp) {
      void window.fraia.quitApp();
      return;
    }
    window.close();
  };
  const checkForUpdates = async () => {
    setUpdateOpen(true);
    setUpdateAction('checking');
    try {
      const status = await window.fraia.checkForUpdates?.();
      if (status) setUpdateStatus(status);
    } catch {
      const status = await window.fraia.updateStatus?.().catch(() => null);
      if (status) setUpdateStatus(status);
    } finally {
      setUpdateAction(null);
    }
  };
  const installUpdate = async () => {
    setUpdateAction('installing');
    try {
      const status = await window.fraia.installUpdate?.();
      if (status) setUpdateStatus(status);
    } catch {
      const status = await window.fraia.updateStatus?.().catch(() => null);
      if (status) setUpdateStatus(status);
    } finally {
      setUpdateAction(null);
    }
  };
  const setUpdateFrequency = async (frequency: UpdateFrequency) => {
    try {
      const status = await window.fraia.setUpdateFrequency?.(frequency);
      if (status) setUpdateStatus(status);
    } catch {
      const status = await window.fraia.updateStatus?.().catch(() => null);
      if (status) setUpdateStatus(status);
    }
  };
  const updateMenuLabel = updateStatus?.phase === 'ready'
    ? 'Restart to Update…'
    : updateStatus?.phase === 'downloading'
      ? 'Downloading Update…'
      : updateStatus?.phase === 'checking'
        ? 'Checking for Updates…'
        : 'Check for Updates…';

  const menus: Array<{ key: string; label: string; groups: MenuItem[][] }> = [
    {
      key: 'fraia',
      label: productName,
      groups: [
        [
          { label: 'Fraia AI…', onSelect: () => setFraiaAiOpen(true) },
          ...(window.fraia.updateStatus ? [{
            label: updateMenuLabel,
            onSelect: () => {
              if (['downloading', 'ready', 'installing'].includes(updateStatus?.phase ?? '')) {
                setUpdateOpen(true);
              } else {
                void checkForUpdates();
              }
            },
          }] : []),
        ],
        [
          { label: `Quit ${productName}`, onSelect: quitApp },
        ],
      ],
    },
    {
      key: 'developer',
      label: 'Developer',
      groups: [
        [
          { label: 'Reload Window', onSelect: reloadWindow, detail: 'Cmd+R' },
          { label: 'Force Reload Window', onSelect: forceReloadWindow, detail: 'Shift+Cmd+R' },
        ],
      ],
    },
  ];

  return (
    <>
      <div
        data-app-menu-frame
        className="relative flex w-full items-center px-1"
        style={{ height: CHROME.menuHeight }}
      >
        <Menubar aria-label="Application menu" className="contents">
          {menus.map((menu) => (
            <MenubarMenu key={menu.key}>
              <MenubarTrigger>{menu.label}</MenubarTrigger>
              <MenubarContent>
                {menu.groups.map((group, groupIndex) => (
                  <Fragment key={`${menu.key}-${groupIndex}`}>
                    {groupIndex > 0 ? <MenubarSeparator /> : null}
                    <MenubarGroup>
                      {group.map((item) => (
                        <MenubarItem key={item.label} disabled={item.disabled} onClick={item.onSelect}>
                          {item.label}
                          {item.detail ? <MenubarShortcut>{item.detail}</MenubarShortcut> : null}
                        </MenubarItem>
                      ))}
                    </MenubarGroup>
                  </Fragment>
                ))}
              </MenubarContent>
            </MenubarMenu>
          ))}
        </Menubar>
        <Separator className="absolute inset-x-0 bottom-0" />
      </div>
      <AiProvidersDialog open={fraiaAiOpen} onOpenChange={setFraiaAiOpen} />
      <UpdateDialog
        checking={updateAction === 'checking' || updateStatus?.phase === 'checking'}
        installing={updateAction === 'installing' || updateStatus?.phase === 'installing'}
        onCheck={() => { void checkForUpdates(); }}
        onInstall={() => { void installUpdate(); }}
        onOpenChange={setUpdateOpen}
        onSetFrequency={(frequency) => { void setUpdateFrequency(frequency); }}
        open={updateOpen}
        status={updateStatus}
      />
    </>
  );
}
