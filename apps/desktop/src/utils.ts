/** Extract a human-readable message from a Tauri AppError or any thrown value. */
export function fmtErr(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return (err as { message: string }).message;
  }
  return String(err);
}
