import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import type { User } from '../api';

interface AuthState {
    token: string | null;
    refreshToken: string | null;
    user: User | null;
    isAuthenticated: boolean;
}

const initialState: AuthState = {
    token: null,
    refreshToken: null,
    user: null,
    isAuthenticated: false
};

function createAuthStore() {
    const { subscribe, set, update } = writable<AuthState>(initialState);

    return {
        subscribe,
        login: (token: string, user: User, refreshToken: string) => {
            if (browser) {
                localStorage.setItem('token', token);
                localStorage.setItem('refreshToken', refreshToken);
                localStorage.setItem('user', JSON.stringify(user));
            }
            set({
                token,
                refreshToken,
                user,
                isAuthenticated: true
            });
        },
        updateTokens: (token: string, refreshToken: string) => {
            if (browser) {
                localStorage.setItem('token', token);
                localStorage.setItem('refreshToken', refreshToken);
            }
            update(state => ({
                ...state,
                token,
                refreshToken
            }));
        },
        logout: () => {
            if (browser) {
                localStorage.removeItem('token');
                localStorage.removeItem('refreshToken');
                localStorage.removeItem('user');
            }
            set(initialState);
            if (browser) {
                window.location.href = '/auth/login';
            }
        },
        initialize: () => {
            if (browser) {
                const token = localStorage.getItem('token');
                const refreshToken = localStorage.getItem('refreshToken');
                const userStr = localStorage.getItem('user');
                
                if (token && userStr) {
                    try {
                        const user = JSON.parse(userStr);
                        set({
                            token,
                            refreshToken: refreshToken || null,
                            user,
                            isAuthenticated: true
                        });
                    } catch {
                        // Invalid user data
                        localStorage.removeItem('token');
                        localStorage.removeItem('refreshToken');
                        localStorage.removeItem('user');
                        set(initialState);
                    }
                }
            }
        }
    };
}

export const auth = createAuthStore();
