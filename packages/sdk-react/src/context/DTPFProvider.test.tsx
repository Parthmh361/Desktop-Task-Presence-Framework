import { renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { useDTPFContext } from './DTPFProvider';

describe('useDTPFContext', () => {
  it('throws when used outside DTPFProvider', () => {
    expect(() => renderHook(() => useDTPFContext())).toThrow(
      'useDTPFContext must be used within DTPFProvider',
    );
  });
});
