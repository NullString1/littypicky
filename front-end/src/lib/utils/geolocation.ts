/**
 * Geolocation utilities for getting user's current position
 */

export interface Coordinates {
  lat: number;
  lng: number;
}

export interface GeolocationOptions {
  fallbackLat?: number;
  fallbackLng?: number;
  timeout?: number;
  maximumAge?: number;
}

const DEFAULT_FALLBACK: Coordinates = {
  lat: 51.5074,  // London
  lng: -0.1278
};

/**
 * Get the user's current location with fallback support
 * @param options Optional configuration for geolocation and fallback
 * @returns Promise resolving to coordinates
 */
export async function getCurrentLocation(
  options: GeolocationOptions = {}
): Promise<Coordinates> {
  const {
    fallbackLat = DEFAULT_FALLBACK.lat,
    fallbackLng = DEFAULT_FALLBACK.lng,
    timeout = 10000,
    maximumAge = 0
  } = options;

  if (!navigator.geolocation) {
    console.warn('Geolocation not supported by browser, using fallback location');
    return { lat: fallbackLat, lng: fallbackLng };
  }

  return new Promise((resolve) => {
    navigator.geolocation.getCurrentPosition(
      (position) => {
        resolve({
          lat: position.coords.latitude,
          lng: position.coords.longitude
        });
      },
      (error) => {
        console.warn('Geolocation denied or failed:', error.message);
        resolve({ lat: fallbackLat, lng: fallbackLng });
      },
      {
        timeout,
        maximumAge,
        enableHighAccuracy: false
      }
    );
  });
}

/**
 * Calculate distance between two coordinates using Haversine formula
 * @param lat1 First latitude
 * @param lon1 First longitude
 * @param lat2 Second latitude
 * @param lon2 Second longitude
 * @returns Formatted distance string (e.g., "500m" or "2.3km")
 */
export function calculateDistance(
  lat1: number,
  lon1: number,
  lat2: number,
  lon2: number
): string {
  const R = 6371; // Radius of the earth in km
  const dLat = deg2rad(lat2 - lat1);
  const dLon = deg2rad(lon2 - lon1);
  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos(deg2rad(lat1)) *
      Math.cos(deg2rad(lat2)) *
      Math.sin(dLon / 2) *
      Math.sin(dLon / 2);
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  const d = R * c; // Distance in km

  if (d < 1) {
    return `${Math.round(d * 1000)}m`;
  }
  return `${d.toFixed(1)}km`;
}

function deg2rad(deg: number): number {
  return deg * (Math.PI / 180);
}

/**
 * Reverse geocode coordinates to address information
 * Uses OpenStreetMap Nominatim API
 */
export async function reverseGeocode(lat: number, lng: number): Promise<{
  city: string;
  country: string;
} | null> {
  try {
    const response = await fetch(
      `https://nominatim.openstreetmap.org/reverse?format=json&lat=${lat}&lon=${lng}`
    );
    const data = await response.json();

    if (data && data.address) {
      return {
        city: data.address.city || data.address.town || data.address.village || data.address.hamlet || '',
        country: data.address.country || ''
      };
    }
    return null;
  } catch (error) {
    console.error('Reverse geocoding failed:', error);
    return null;
  }
}
