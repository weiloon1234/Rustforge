function matchPattern(pattern: string, value: string): boolean {
  if (!pattern.endsWith(".*")) return false;
  const prefix = pattern.slice(0, -2);
  if (!prefix) return false;
  return value === prefix || value.startsWith(prefix + ".");
}

function manageImpliesRead(granted: string, required: string): boolean {
  const gi = granted.lastIndexOf(".");
  const ri = required.lastIndexOf(".");
  if (gi === -1 || ri === -1) return false;
  return (
    granted.slice(0, gi) === required.slice(0, ri) &&
    granted.slice(gi + 1) === "manage" &&
    required.slice(ri + 1) === "read"
  );
}

export function permissionMatches(granted: string, required: string): boolean {
  const g = granted.trim();
  const r = required.trim();
  if (!g || !r) return false;
  if (g === "*" || r === "*" || g === r) return true;
  if (manageImpliesRead(g, r)) return true;
  return matchPattern(g, r) || matchPattern(r, g);
}

export function hasPermission(
  scopes: readonly string[],
  required: string,
): boolean {
  return scopes.some((scope) => permissionMatches(scope, required));
}

export function hasAnyPermission(
  scopes: readonly string[],
  required: readonly string[],
): boolean {
  if (required.length === 0) return true;
  return required.some((permission) => hasPermission(scopes, permission));
}
