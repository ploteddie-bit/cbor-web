import { useCallback, useEffect, useRef, useState } from "react";
import { CBORWebClient } from "../typescript/cborweb";

type CBORValue = Record<string, unknown> | unknown[] | string | number | boolean | null | Uint8Array;

interface UseCBORWebReturn {
  manifest: CBORValue | null;
  pages: Record<string, CBORValue>;
  bundle: CBORValue | null;
  loading: boolean;
  error: Error | null;
  fetchPage: (path: string) => Promise<CBORValue | null>;
  fetchBundle: () => Promise<CBORValue | null>;
}

export function useCBORWeb(baseUrl: string): UseCBORWebReturn {
  const [manifest, setManifest] = useState<CBORValue | null>(null);
  const [pages, setPages] = useState<Record<string, CBORValue>>({});
  const [bundle, setBundle] = useState<CBORValue | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const clientRef = useRef<CBORWebClient | null>(null);
  const manifestRef = useRef<CBORValue | null>(null);

  useEffect(() => {
    const client = new CBORWebClient(baseUrl);
    clientRef.current = client;

    setLoading(true);
    setError(null);

    client
      .manifest()
      .then((m) => {
        manifestRef.current = m;
        setManifest(m);
      })
      .catch((e) => {
        setError(e instanceof Error ? e : new Error(String(e)));
      })
      .finally(() => setLoading(false));
  }, [baseUrl]);

  const fetchPage = useCallback(async (path: string): Promise<CBORValue | null> => {
    if (!clientRef.current) return null;
    try {
      setError(null);
      const data = await clientRef.current.page(path);
      setPages((prev) => ({ ...prev, [path]: data }));
      return data;
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      setError(err);
      return null;
    }
  }, []);

  const fetchBundle = useCallback(async (): Promise<CBORValue | null> => {
    if (!clientRef.current) return null;
    try {
      setError(null);
      const data = await clientRef.current.bundle();
      setBundle(data);
      return data;
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      setError(err);
      return null;
    }
  }, []);

  return { manifest, pages, bundle, loading, error, fetchPage, fetchBundle };
}
