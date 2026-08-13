import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarRadioGroup,
  MenubarRadioItem,
  MenubarSeparator,
  MenubarShortcut,
  MenubarSub,
  MenubarSubContent,
  MenubarSubTrigger,
  MenubarTrigger,
} from '@/components/ui/menubar';
import { Separator } from '@/components/ui/separator';
import { UpdateDialog } from '@/components/updates/UpdateDialog';
import {
  UPDATE_FREQUENCY_LABELS,
  type UpdateFrequency,
  type UpdateStatus,
} from '@/lib/updateStatus';
import { useEffect, useState } from 'react';
import { CHROME } from './chromeMetrics';

export function AppMenuBar() {
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

  return (
    <>
      <div
        data-app-menu-frame
        className="relative flex w-full items-center px-1"
        style={{ height: CHROME.menuHeight }}
      >
        <Menubar aria-label="Application menu" className="contents">
          <MenubarMenu>
            <MenubarTrigger>{productName}</MenubarTrigger>
            <MenubarContent className="w-max min-w-max whitespace-nowrap">
              {window.fraia.updateStatus ? (
                <>
                  <MenubarGroup>
                    <MenubarItem
                      onClick={() => {
                        if (['downloading', 'ready', 'installing'].includes(updateStatus?.phase ?? '')) {
                          setUpdateOpen(true);
                        } else {
                          void checkForUpdates();
                        }
                      }}
                    >
                      {updateMenuLabel}
                    </MenubarItem>
                    <MenubarSub>
                      <MenubarSubTrigger disabled={!updateStatus?.enabled}>
                        Check Automatically
                      </MenubarSubTrigger>
                      <MenubarSubContent className="w-max min-w-max whitespace-nowrap">
                        <MenubarRadioGroup
                          value={updateStatus?.frequency ?? 'daily'}
                          onValueChange={(value) => {
                            if (typeof value === 'string' && value in UPDATE_FREQUENCY_LABELS) {
                              void setUpdateFrequency(value as UpdateFrequency);
                            }
                          }}
                        >
                          {(Object.entries(UPDATE_FREQUENCY_LABELS) as Array<[UpdateFrequency, string]>).map(([value, label]) => (
                            <MenubarRadioItem key={value} value={value}>
                              {label}
                            </MenubarRadioItem>
                          ))}
                        </MenubarRadioGroup>
                      </MenubarSubContent>
                    </MenubarSub>
                  </MenubarGroup>
                  <MenubarSeparator />
                </>
              ) : null}
              <MenubarGroup>
                <MenubarItem onClick={quitApp}>Quit {productName}</MenubarItem>
              </MenubarGroup>
            </MenubarContent>
          </MenubarMenu>
          <MenubarMenu>
            <MenubarTrigger>File</MenubarTrigger>
            <MenubarContent className="w-max min-w-max whitespace-nowrap">
              <MenubarGroup>
                <MenubarItem onClick={() => window.dispatchEvent(new CustomEvent('fraia:save-project', { detail: { saveAs: false } }))}>
                  Save <MenubarShortcut>⌘S</MenubarShortcut>
                </MenubarItem>
                <MenubarItem onClick={() => window.dispatchEvent(new CustomEvent('fraia:save-project', { detail: { saveAs: true } }))}>
                  Save As… <MenubarShortcut>⇧⌘S</MenubarShortcut>
                </MenubarItem>
              </MenubarGroup>
            </MenubarContent>
          </MenubarMenu>
        </Menubar>
        <Separator className="absolute inset-x-0 bottom-0" />
      </div>
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
