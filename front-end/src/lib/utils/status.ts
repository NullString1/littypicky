/**
 * Status-related utilities for report statuses
 */

export type ReportStatus = "pending" | "claimed" | "cleared" | "verified";

/**
 * Get Tailwind CSS classes for a report status badge
 * @param status The report status
 * @returns CSS class string for the status badge
 */
export function getStatusColor(status: string): string {
  switch (status) {
    case "pending":
      return "bg-red-100 text-red-800";
    case "claimed":
      return "bg-yellow-100 text-yellow-800";
    case "cleared":
      return "bg-green-100 text-green-800";
    case "verified":
      return "bg-blue-100 text-blue-800";
    default:
      return "bg-slate-100 text-slate-800";
  }
}

/**
 * Get a human-readable label for a status
 * @param status The report status
 * @returns Formatted status label
 */
export function getStatusLabel(status: string): string {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

/**
 * Check if a status allows claiming
 * @param status The report status
 * @returns True if the report can be claimed
 */
export function canClaim(status: string): boolean {
  return status === "pending";
}

/**
 * Check if a status allows clearing
 * @param status The report status
 * @returns True if the report can be marked as cleared
 */
export function canClear(status: string): boolean {
  return status === "claimed";
}

/**
 * Check if a status allows verification
 * @param status The report status
 * @returns True if the report can be verified
 */
export function canVerify(status: string): boolean {
  return status === "cleared";
}
