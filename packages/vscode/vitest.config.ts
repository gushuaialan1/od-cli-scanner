import { defineConfig } from 'vitest/config';
import * as path from 'path';

export default defineConfig({
  resolve: {
    alias: {
      // 'vscode' is only available inside the VS Code extension host;
      // alias it to a lightweight mock for unit tests.
      vscode: path.resolve(__dirname, 'src/test/__mocks__/vscode.ts'),
    },
  },
  test: {
    environment: 'node',
    include: ['src/test/**/*.test.ts'],
  },
});
