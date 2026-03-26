export const WINDOWS_URL = import.meta.env.VITE_DOWNLOAD_WINDOWS_URL as string
export const LINUX_URL = import.meta.env.VITE_DOWNLOAD_LINUX_URL as string

export type OS = 'windows' | 'linux' | 'mac' | 'unknown'

export function getOS(): OS {
  if (typeof navigator === 'undefined') return 'unknown'
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('mac os') || ua.includes('macintosh')) return 'mac'
  if (ua.includes('win')) return 'windows'
  if (ua.includes('linux')) return 'linux'
  return 'unknown'
}
