import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import type { User } from '../api';

interface AuthState {
    token: string | null;
    user: User | null;
    isAuthenticated: boolean;
}

const initialState: AuthState = {
    token: null,
    user: null,
    isAuthenticated: false
};

function createAuthStore() {
    const { subscribe, set, update } = writable<AuthState>(initialState);

    return {
        subscribe,
        login: (token: string, user: User) => {
            if (browser) {
                localStorage.setItem('token', token);
                localStorage.setItem('user', JSON.stringify(user));
            }
            set({
                token,
                user,
                isAuthenticated: true
            });
        },
        logout: () => {
            if (browser) {
                localStorage.removeItem('token');
                localStorage.removeItem('user');
            }
            set(initialState);
        },
        initialize: () => {
            if (browser) {
                const token = localStorage.getItem('token');
                const userStr = localStorage.getItem('user');
                
                if (token && userStr) {
                    try {
                        const user = JSON.parse(userStr);
                        set({
                            token,
                            user,
                            isAuthenticated: true
                        });
                    } catch {
                        // Invalid user data
                        localStorage.removeItem('token');
                        localStorage.removeItem('user');
                        set(initialState);
                    }
                }
            }
        }
    };
}

export const auth = createAuthStore();
