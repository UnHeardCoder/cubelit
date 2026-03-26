const rawWindows = import.meta.env.VITE_DOWNLOAD_WINDOWS_URL
const rawLinux = import.meta.env.VITE_DOWNLOAD_LINUX_URL

if (!rawWindows) throw new Error('VITE_DOWNLOAD_WINDOWS_URL is not set')
if (!rawLinux) throw new Error('VITE_DOWNLOAD_LINUX_URL is not set')

export const WINDOWS_URL: string = rawWindows
export const LINUX_URL: string = rawLinux

export type OS = 'windows' | 'linux' | 'mac' | 'unknown'

export function getOS(): OS {
  if (typeof navigator === 'undefined') return 'unknown'
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('mac os') || ua.includes('macintosh')) return 'mac'
  if (ua.includes('win')) return 'windows'
  if (ua.includes('linux')) return 'linux'
  return 'unknown'
}
