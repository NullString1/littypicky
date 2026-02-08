import type { components } from './api-types';
import { get } from 'svelte/store';
import { auth } from '$lib/stores/auth';

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
export type RefreshTokenRequest = components['schemas']['RefreshTokenRequest'];
export type RefreshTokenResponse = components['schemas']['RefreshTokenResponse'];
export type FeedPostResponse = components['schemas']['FeedPostResponse'];
export type FeedComment = components['schemas']['FeedComment'];
export type FeedCommentResponse = components['schemas']['FeedCommentResponse'];
export type CreateFeedPostRequest = components['schemas']['CreateFeedPostRequest'];
export type UpdateFeedPostRequest = components['schemas']['UpdateFeedPostRequest'];
export type CreateFeedCommentRequest = components['schemas']['CreateFeedCommentRequest'];
export type UpdateFeedCommentRequest = components['schemas']['UpdateFeedCommentRequest'];
export type ForgotPasswordRequest = components['schemas']['ForgotPasswordRequest'];
export type ResetPasswordRequest = components['schemas']['ResetPasswordRequest'];
export type MessageResponse = components['schemas']['MessageResponse'];

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

    // Use provided token or fall back to store token
    let currentToken = token;
    if (!currentToken) {
        const state = get(auth);
        if (state.token) {
            currentToken = state.token;
        }
    }

    if (currentToken) {
        headers['Authorization'] = `Bearer ${currentToken}`;
    }

    const options: RequestInit = {
        method,
        headers,
    };

    if (body) {
        options.body = JSON.stringify(body);
    }

    const response = await fetch(`${API_BASE}${path}`, options);

    if (response.status === 401) {
        // Prevent infinite loop for refresh endpoint or login
        if (path === '/auth/refresh' || path === '/auth/login') {
             throw new ApiError('Unauthorized', 401);
        }

        const state = get(auth);
        const refreshToken = state.refreshToken;
        
        if (refreshToken) {
            try {
                // Try to refresh the token
                const refreshResponse = await fetch(`${API_BASE}/auth/refresh`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({ refresh_token: refreshToken })
                });

                if (refreshResponse.ok) {
                    const data = await refreshResponse.json();
                    // Update store with new access token (and refresh token if rotated)
                    // Note: API might not return a new refresh token, so keep the old one if needed
                    const newAccessToken = data.access_token;
                    const newRefreshToken = data.refresh_token || refreshToken;
                    
                    auth.updateTokens(newAccessToken, newRefreshToken);
                    
                    // Retry original request with new token
                    return request<T>(method, path, body, newAccessToken);
                } else {
                    // Refresh failed (e.g. refresh token expired)
                    auth.logout();
                    throw new ApiError('Session expired', 401);
                }
            } catch (e) {
                // Network error or other issue during refresh
                auth.logout();
                throw e;
            }
        } else {
             // No refresh token available
             auth.logout();
        }
    }

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
        refreshToken: (data: RefreshTokenRequest) => request<RefreshTokenResponse>('POST', '/auth/refresh', data),
        getMe: (token: string) => request<User>('GET', '/users/me', undefined, token),
        forgotPassword: (data: ForgotPasswordRequest) => request<MessageResponse>('POST', '/auth/forgot-password', data),
        resetPassword: (data: ResetPasswordRequest) => request<MessageResponse>('POST', '/auth/reset-password', data),
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
    },
    feed: {
        getAll: (offset: number = 0, limit: number = 20, token?: string) => 
            request<FeedPostResponse[]>('GET', `/feed?offset=${offset}&limit=${limit}`, undefined, token),
        getById: (id: string, token?: string) => 
            request<FeedPostResponse>('GET', `/feed/${id}`, undefined, token),
        create: (data: CreateFeedPostRequest, token?: string) => 
            request<FeedPostResponse>('POST', '/feed', data, token),
        update: (id: string, data: UpdateFeedPostRequest, token?: string) => 
            request<FeedPostResponse>('PATCH', `/feed/${id}`, data, token),
        delete: (id: string, token?: string) => 
            request<void>('DELETE', `/feed/${id}`, undefined, token),
        comments: {
            create: (postId: string, data: CreateFeedCommentRequest, token?: string) => 
                request<FeedComment>('POST', `/feed/${postId}/comments`, data, token),
            get: (postId: string, token?: string) => 
                request<FeedCommentResponse[]>('GET', `/feed/${postId}/comments`, undefined, token),
            update: (commentId: string, data: UpdateFeedCommentRequest, token?: string) => 
                request<FeedComment>('PATCH', `/feed/comments/${commentId}`, data, token),
            delete: (commentId: string, token?: string) => 
                request<void>('DELETE', `/feed/comments/${commentId}`, undefined, token),
        },
        likes: {
            create: (postId: string, token?: string) => 
                request<void>('POST', `/feed/${postId}/like`, {}, token),
            delete: (postId: string, token?: string) => 
                request<void>('DELETE', `/feed/${postId}/like`, undefined, token),
        }
    }
};

