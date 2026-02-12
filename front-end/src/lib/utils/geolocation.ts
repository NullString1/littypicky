/**
 * Geolocation utilities for getting user's current position
 */

export interface Coordinates {
  lat: number;
  lng: number;
  accuracy?: number;
}

export interface GeolocationOptions {
  fallbackLat?: number;
  fallbackLng?: number;
  timeout?: number;
  maximumAge?: number;
  enableHighAccuracy?: boolean;
  minAccuracyMeters?: number;
  maxAttempts?: number;
}

const DEFAULT_FALLBACK: Coordinates = {
  lat: 51.5074, // London
  lng: -0.1278,
};

/**
 * Get the user's current location with fallback support
 * @param options Optional configuration for geolocation and fallback
 * @returns Promise resolving to coordinates
 */
export async function getCurrentLocation(
  options: GeolocationOptions = {},
): Promise<Coordinates> {
  const {
    fallbackLat = DEFAULT_FALLBACK.lat,
    fallbackLng = DEFAULT_FALLBACK.lng,
    timeout = 10000,
    maximumAge = 0,
    enableHighAccuracy = false,
    minAccuracyMeters,
    maxAttempts = 2,
  } = options;

  if (!navigator.geolocation) {
    console.warn(
      "Geolocation not supported by browser, using fallback location",
    );
    return { lat: fallbackLat, lng: fallbackLng };
  }

  return new Promise((resolve) => {
    let attempts = 0;

    const requestPosition = () => {
      attempts += 1;
      navigator.geolocation.getCurrentPosition(
        (position) => {
          const coords = {
            lat: position.coords.latitude,
            lng: position.coords.longitude,
            accuracy: position.coords.accuracy,
          };

          if (
            minAccuracyMeters &&
            typeof coords.accuracy === "number" &&
            coords.accuracy > minAccuracyMeters &&
            attempts < maxAttempts
          ) {
            requestPosition();
            return;
          }

          resolve(coords);
        },
        (error) => {
          console.warn("Geolocation denied or failed:", error.message);
          resolve({ lat: fallbackLat, lng: fallbackLng });
        },
        {
          timeout,
          maximumAge,
          enableHighAccuracy,
        },
      );
    };

    requestPosition();
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
  lon2: number,
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
export async function reverseGeocode(
  lat: number,
  lng: number,
): Promise<{
  city: string;
  region?: string;
  country: string;
} | null> {
  try {
    const nominatimResponse = await fetch(
      `https://nominatim.openstreetmap.org/reverse?format=json&lat=${lat}&lon=${lng}&email=hello@littypicky.app`,
    );
    const nominatimData = await nominatimResponse.json();

    if (nominatimData && nominatimData.address) {
      return {
        city:
          nominatimData.address.city ||
          nominatimData.address.town ||
          nominatimData.address.village ||
          nominatimData.address.hamlet ||
          "",
        region:
          nominatimData.address.state ||
          nominatimData.address.region ||
          nominatimData.address.county ||
          "",
        country: nominatimData.address.country || "",
      };
    }

    return null;
  } catch (error) {
    console.error("Reverse geocoding failed:", error);
    return null;
  }
}

/**
 * Get coordinates from a user's profile location
 */
export async function getProfileLocationCoordinates(
  user: { city?: string; country?: string } | null,
): Promise<Coordinates | null> {
  if (!user?.city || !user?.country || user.city === "Unknown") {
    return null;
  }

  const cacheKey = `lp_geo_${user.city}_${user.country}`;
  try {
    const cached = localStorage.getItem(cacheKey);
    if (cached) {
      return JSON.parse(cached);
    }
  } catch {}

  const citySimple = user.city.split(",")[0].trim();
  const queries = [
    `${citySimple}, ${user.country}`, // Priority 1: "Plymouth, United Kingdom"
    `${user.city}, ${user.country}`, // Priority 2: "Plymouth, England, United Kingdom"
    user.city, // Priority 3: "Plymouth, England"
    citySimple, // Priority 4: "Plymouth"
  ];

  // Remove duplicates and empty strings
  const uniqueQueries = [
    ...new Set(queries.filter((q) => q && q.trim().length > 0)),
  ];

  for (const query of uniqueQueries) {
    try {
      const res = await fetch(
        `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(query)}&count=1&language=en&format=json`,
      );
      const data = await res.json();

      if (data.results?.[0]) {
        const result = {
          lat: data.results[0].latitude,
          lng: data.results[0].longitude,
        };
        try {
          localStorage.setItem(cacheKey, JSON.stringify(result));
        } catch {}
        return result;
      }
    } catch (e) {
      console.warn(`Geocoding failed for query "${query}":`, e);
    }
  }

  return null;
}
