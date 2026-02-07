import type { components } from './api-types';

export type User = components['schemas']['UserResponse'];
export type LoginRequest = components['schemas']['LoginRequest'];
export type RegisterRequest = components['schemas']['RegisterRequest'];
export type AuthTokens = components['schemas']['AuthTokens'];
export type Report = components['schemas']['ReportResponse'];
export type CreateReportRequest = components['schemas']['CreateReportRequest'];
export type ClearReportRequest = components['schemas']['ClearReportRequest'];
export type CreateVerificationRequest = components['schemas']['CreateVerificationRequest'];
export type VerificationResponse = components['schemas']['VerificationResponse'];
export type LeaderboardEntry = components['schemas']['LeaderboardEntry'];
export type UserScoreRecord = components['schemas']['UserScoreRecord'];
export type UpdateUserRequest = components['schemas']['UpdateUserRequest'];

const API_BASE = '/api';

class ApiError extends Error {
    status: number;
    constructor(message: string, status: number) {
        super(message);
        this.status = status;
    }
}

async function request<T>(
    method: string,
    path: string,
    body?: unknown,
    token?: string
): Promise<T> {
    const headers: HeadersInit = {
        'Content-Type': 'application/json',
    };

    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }

    const options: RequestInit = {
        method,
        headers,
    };

    if (body) {
        options.body = JSON.stringify(body);
    }

    const response = await fetch(`${API_BASE}${path}`, options);

    if (!response.ok) {
        let errorMessage = 'An unknown error occurred';
        try {
            const errorData = await response.json();
             // Adjust based on how backend sends errors, assuming usually a message field or string
            errorMessage = errorData.message || errorData.error || JSON.stringify(errorData);
        } catch {
            errorMessage = response.statusText;
        }
        throw new ApiError(errorMessage, response.status);
    }

    // Handle 204 No Content
    if (response.status === 204) {
        return {} as T;
    }

    try {
        return await response.json();
    } catch {
        return {} as T;
    }
}

export const api = {
    auth: {
        login: (data: LoginRequest) => request<AuthTokens>('POST', '/auth/login', data),
        register: (data: RegisterRequest) => request<{ message: string }>('POST', '/auth/register', data),
        verifyEmail: (token: string) => request<AuthTokens>('POST', '/auth/verify-email', { token }),
        getMe: (token: string) => request<User>('GET', '/users/me', undefined, token),
    },
    users: {
        updateMe: (data: UpdateUserRequest, token: string) => request<User>('PATCH', '/users/me', data, token),
        getMyScore: (token: string) => request<UserScoreRecord>('GET', '/users/me/score', undefined, token),
    },
    reports: {
        create: (data: CreateReportRequest, token: string) => request<Report>('POST', '/reports', data, token),
        getNearby: (latitude: number, longitude: number, radius_km: number, token: string) => 
            request<Report[]>('GET', `/reports/nearby?latitude=${latitude}&longitude=${longitude}&radius_km=${radius_km}`, undefined, token),
        getVerificationQueue: (latitude: number, longitude: number, radius_km: number, token: string) => 
            request<Report[]>('GET', `/reports/verification-queue?latitude=${latitude}&longitude=${longitude}&radius_km=${radius_km}`, undefined, token),
        getMyReports: (token: string) => request<Report[]>('GET', '/reports/my-reports', undefined, token),
        getMyClears: (token: string) => request<Report[]>('GET', '/reports/my-clears', undefined, token),
        getById: (id: string, token: string) => request<Report>('GET', `/reports/${id}`, undefined, token),
        claim: (id: string, token: string) => request<Report>('POST', `/reports/${id}/claim`, {}, token),
        clear: (id: string, data: ClearReportRequest, token: string) => request<Report>('POST', `/reports/${id}/clear`, data, token),
        verify: (id: string, data: CreateVerificationRequest, token: string) => request<VerificationResponse>('POST', `/reports/${id}/verify`, data, token),
        getVerifications: (id: string, token: string) => request<VerificationResponse[]>('GET', `/reports/${id}/verifications`, undefined, token),
    },
    leaderboards: {
        getGlobal: (period: string = 'weekly', token: string) => request<LeaderboardEntry[]>('GET', `/leaderboards?period=${period}`, undefined, token),
        getCity: (city: string, period: string = 'weekly', token: string) => request<LeaderboardEntry[]>('GET', `/leaderboards/city/${city}?period=${period}`, undefined, token),
        getCountry: (country: string, period: string = 'weekly', token: string) => request<LeaderboardEntry[]>('GET', `/leaderboards/country/${country}?period=${period}`, undefined, token),
    }
};
