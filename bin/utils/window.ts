import logger from '@/options/logger';

export const MAIN_WINDOW_LABEL = 'pake';

const LABEL_PATTERN = /^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$/;

export interface ParsedWindowSpec {
  label: string;
  path: string;
}

export function parseWindowSpec(spec: string): ParsedWindowSpec {
  const eqIndex = spec.indexOf('=');
  if (eqIndex <= 0) {
    throw new Error(
      `Invalid --window format "${spec}". Expected <label>=<path>, e.g. camera=/camera`,
    );
  }

  const label = spec.slice(0, eqIndex).trim();
  const path = normalizeMsysMangledPath(spec.slice(eqIndex + 1).trim());

  if (!path) {
    throw new Error(
      `Invalid --window format "${spec}". Path must not be empty.`,
    );
  }

  return { label, path };
}

/**
 * Git Bash (MSYS) rewrites Unix-style CLI args: `/live` becomes
 * `C:/Program Files/Git/live` before Node sees them. Recover the route.
 */
export function normalizeMsysMangledPath(path: string): string {
  const decoded = decodeURIComponent(path).replace(/\\/g, '/');
  const msysMatch = decoded.match(
    /^[a-zA-Z]:\/(?:Program Files|Program%20Files)\/Git\/(.+)$/i,
  );
  if (!msysMatch) {
    return path;
  }
  const route = msysMatch[1];
  return route.startsWith('/') ? route : `/${route}`;
}

export function isWindowsFilePath(path: string): boolean {
  return /^[a-zA-Z]:[/\\]/.test(path) && !/^https?:/i.test(path);
}

export function isValidWindowLabel(label: string): boolean {
  return LABEL_PATTERN.test(label);
}

export function resolveWindowUrl(baseUrl: string, path: string): string {
  if (path.startsWith('/') || path.startsWith('./') || path.startsWith('../')) {
    return new URL(path, baseUrl).href;
  }

  try {
    return new URL(path).href;
  } catch {
    return new URL(path, baseUrl).href;
  }
}

export function getUrlOrigin(url: string): string {
  return new URL(url).origin;
}

export interface ValidateWindowSpecsOptions {
  exitOnError?: boolean;
}

export function validateWindowSpecs(
  specs: string[],
  baseUrl: string,
  options: ValidateWindowSpecsOptions = {},
): ParsedWindowSpec[] {
  const { exitOnError = true } = options;
  const fail = (message: string): never => {
    logger.error(message);
    if (exitOnError) {
      process.exit(1);
    }
    throw new Error(message);
  };

  if (specs.length === 0) {
    return [];
  }

  const baseOrigin = getUrlOrigin(baseUrl);
  const seen = new Set<string>();
  const parsed: ParsedWindowSpec[] = [];

  for (const spec of specs) {
    let entry: ParsedWindowSpec;
    try {
      entry = parseWindowSpec(spec);
    } catch (error) {
      return fail(error instanceof Error ? error.message : String(error));
    }

    if (entry.label === MAIN_WINDOW_LABEL) {
      fail(
        `Window label "${MAIN_WINDOW_LABEL}" is reserved for the main window. Choose another label.`,
      );
    }

    if (!isValidWindowLabel(entry.label)) {
      fail(
        `Invalid window label "${entry.label}". Labels must match [a-zA-Z0-9-]+ and cannot start or end with a hyphen.`,
      );
    }

    if (seen.has(entry.label)) {
      fail(`Duplicate window label "${entry.label}" in --window options.`);
    }
    seen.add(entry.label);

    if (isWindowsFilePath(entry.path)) {
      fail(
        `Window path "${entry.path}" looks like a Windows file path (Git Bash may have rewritten /route). ` +
          'Use MSYS_NO_PATHCONV=1 before the command, quote the value (--window "live=/live"), ' +
          'or pass a full URL (--window live=https://example.com/live).',
      );
    }

    const resolved = resolveWindowUrl(baseUrl, entry.path);
    try {
      const resolvedOrigin = getUrlOrigin(resolved);
      if (resolvedOrigin !== baseOrigin) {
        logger.warn(
          `✼ Window "${entry.label}" URL origin (${resolvedOrigin}) differs from main URL origin (${baseOrigin}). Session/CORS issues may occur.`,
        );
      }
    } catch {
      fail(`Invalid path for window "${entry.label}": ${entry.path}`);
    }

    parsed.push(entry);
  }

  return parsed;
}

export function collectWindowLabels(
  mainLabel: string,
  extraSpecs: ParsedWindowSpec[],
): string[] {
  return [mainLabel, ...extraSpecs.map((spec) => spec.label)];
}
