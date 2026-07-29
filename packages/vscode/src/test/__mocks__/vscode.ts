// Minimal mock of the 'vscode' module for unit tests.
// Only the surface used by the modules under test is provided.
// Modules under test only use `vscode` for types (e.g. Memento),
// so an empty-ish export surface is sufficient at runtime.

export class TreeItem {
  label?: string;
  collapsibleState?: number;
  constructor(label?: string, collapsibleState?: number) {
    this.label = label;
    this.collapsibleState = collapsibleState;
  }
}

export enum TreeItemCollapsibleState {
  None = 0,
  Collapsed = 1,
  Expanded = 2,
}

export class EventEmitter<T> {
  private listeners: ((e: T) => void)[] = [];
  event = (listener: (e: T) => void) => {
    this.listeners.push(listener);
    return { dispose: () => undefined };
  };
  fire(data: T): void {
    for (const l of this.listeners) {
      l(data);
    }
  }
  dispose(): void {
    this.listeners = [];
  }
}

export const window = {
  createStatusBarItem: () => ({
    show: () => undefined,
    hide: () => undefined,
    dispose: () => undefined,
  }),
  showQuickPick: async () => undefined,
  showInformationMessage: async () => undefined,
  showWarningMessage: async () => undefined,
  showErrorMessage: async () => undefined,
  createTerminal: () => ({ show: () => undefined, sendText: () => undefined }),
};

export const commands = {
  registerCommand: () => ({ dispose: () => undefined }),
  executeCommand: async () => undefined,
};

export const workspace = {
  getConfiguration: () => ({ get: (_key: string, def?: unknown) => def }),
  workspaceFolders: undefined,
};

export enum StatusBarAlignment {
  Left = 1,
  Right = 2,
}

export class ThemeIcon {
  constructor(public readonly id: string) {}
}

export class Uri {
  static file(path: string): Uri {
    const u = new Uri();
    (u as { fsPath: string }).fsPath = path;
    return u;
  }
  readonly fsPath: string = '';
}
