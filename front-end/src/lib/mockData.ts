// Mock Data for LittyPicky Frontend

export interface User {
    id: string;
    name: string;
    handle: string;
    tier: 'Novice' | 'Picker' | 'Verifier' | 'Legend';
    points: number;
    reports: number;
    cleanups: number;
    avatar: string; // URL or placeholder color
    badges: Badge[];
}

export interface Badge {
    id: string;
    name: string;
    icon: string;
    description: string;
}

export interface Report {
    id: string;
    reporterId: string;
    locationName: string;
    coordinates: { lat: number; lng: number };
    photoUrl: string; // Placeholder
    description: string;
    status: 'OPEN' | 'CLEANED' | 'VERIFIED';
    timestamp: Date;
    distance?: string; // Calculated on client usually, mock for now
}

export interface VerificationRequest {
    id: string;
    reportId: string;
    cleanerId: string;
    beforePhotoUrl: string;
    afterPhotoUrl: string;
    timestamp: Date;
    cleanerName: string;
}

// --- DATA ---

export const currentUser: User = {
    id: 'u1',
    name: 'Alex Rivera',
    handle: '@arivera',
    tier: 'Verifier',
    points: 1250,
    reports: 12,
    cleanups: 45,
    avatar: 'bg-indigo-500',
    badges: [
        { id: 'b1', name: 'First Pick', icon: '🌱', description: 'Completed your first cleanup' },
        { id: 'b2', name: 'Weekend Warrior', icon: '⚔️', description: 'Cleaned up on 5 weekends' },
        { id: 'b3', name: 'Verifier', icon: '🛡️', description: 'Unlocked verification privileges' }
    ]
};

export const activeReports: Report[] = [
    {
        id: 'r1',
        reporterId: 'u2',
        locationName: 'Central Park, North Entrance',
        coordinates: { lat: 40.796, lng: -73.958 },
        photoUrl: 'bg-slate-300',
        description: 'Pile of plastic bottles near the bench.',
        status: 'OPEN',
        timestamp: new Date(Date.now() - 3600000), // 1 hour ago
        distance: '0.4 mi'
    },
    {
        id: 'r2',
        reporterId: 'u3',
        locationName: 'Main St. Bus Stop',
        coordinates: { lat: 40.750, lng: -73.990 },
        photoUrl: 'bg-slate-300',
        description: 'Overflowing trash can spilled onto sidewalk.',
        status: 'OPEN',
        timestamp: new Date(Date.now() - 7200000), // 2 hours ago
        distance: '1.2 mi'
    },
    {
        id: 'r3',
        reporterId: 'u4',
        locationName: 'River Walk Trail',
        coordinates: { lat: 40.710, lng: -74.010 },
        photoUrl: 'bg-slate-300',
        description: 'Fast food bags scattered along the path.',
        status: 'OPEN',
        timestamp: new Date(Date.now() - 86400000), // 1 day ago
        distance: '3.5 mi'
    }
];

export const verificationQueue: VerificationRequest[] = [
    {
        id: 'v1',
        reportId: 'r5',
        cleanerId: 'u5',
        cleanerName: 'Sarah Jenkins',
        beforePhotoUrl: 'bg-red-100', // Red tint for "messy"
        afterPhotoUrl: 'bg-green-100', // Green tint for "clean"
        timestamp: new Date()
    },
    {
        id: 'v2',
        reportId: 'r6',
        cleanerId: 'u6',
        cleanerName: 'Mike Chen',
        beforePhotoUrl: 'bg-red-100',
        afterPhotoUrl: 'bg-red-100', // Suspiciously same photo?
        timestamp: new Date()
    }
];

export const communityPosts = [
    {
        id: 'p1',
        user: 'Sarah Jenkins',
        userTier: 'Picker',
        avatar: 'bg-purple-500',
        image: 'bg-green-200',
        content: 'Just cleared 3 bags of trash from the beach! Feels good to see the sand again. 🏖️',
        likes: 24,
        comments: 3,
        time: '2h ago'
    },
    {
        id: 'p2',
        user: 'Alex Rivera',
        userTier: 'Verifier',
        avatar: 'bg-indigo-500',
        image: 'bg-green-200',
        content: 'Verification queue is empty! Great work everyone this weekend.',
        likes: 56,
        comments: 12,
        time: '5h ago'
    }
];
