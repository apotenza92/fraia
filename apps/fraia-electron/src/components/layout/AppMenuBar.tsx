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
import { AiProvidersDialog } from '@/components/ai/AiProvidersDialog';
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

  const menus: Array<{ key: string; label: string; groups: MenuItem[][] }> = [
    {
      key: 'fraia',
      label: productName,
      groups: [
        [
          { label: 'Fraia AI…', onSelect: () => setFraiaAiOpen(true) },
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
    <Menubar aria-label="Application menu" className="rounded-none border-0 border-b px-2 shadow-none" style={{ height: CHROME.menuHeight }}>
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
    <AiProvidersDialog open={fraiaAiOpen} onOpenChange={setFraiaAiOpen} />
    </>
  );
}
