import { describe, it, expect } from 'vitest';
import { ScannerError } from '../types';

describe('ScannerError', () => {
  it('carries code, message and name', () => {
    const err = new ScannerError('TIMEOUT', 'scan timed out');
    expect(err).toBeInstanceOf(Error);
    expect(err.name).toBe('ScannerError');
    expect(err.code).toBe('TIMEOUT');
    expect(err.message).toBe('scan timed out');
    expect(err.cause).toBeUndefined();
  });

  it('retains an optional cause', () => {
    const cause = new Error('spawn failed');
    const err = new ScannerError('BINARY_NOT_FOUND', 'not found', cause);
    expect(err.cause).toBe(cause);
  });
});
