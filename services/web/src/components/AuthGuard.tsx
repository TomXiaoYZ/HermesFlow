'use client';

import { useEffect, useState, useCallback } from 'react';
import { useRouter, usePathname } from 'next/navigation';

const INACTIVITY_LIMIT = 3600 * 1000; // 1 Hour

export default function AuthGuard({ children }: { children: React.ReactNode }) {
    const router = useRouter();
    const pathname = usePathname();
    const [authorized, setAuthorized] = useState(false);

    const logout = useCallback(() => {
        localStorage.removeItem('token');
        localStorage.removeItem('lastActivity');
        setAuthorized(false);
        router.push('/login');
    }, [router]);

    useEffect(() => {
        // Allow public access to login
        if (pathname === '/login') {
            setAuthorized(true);
            return;
        }

        const token = localStorage.getItem('token');
        if (!token) {
            router.push('/login');
            return;
        }

        // Check inactivity
        const lastActivity = localStorage.getItem('lastActivity');
        const now = Date.now();
        if (lastActivity && (now - parseInt(lastActivity, 10) > INACTIVITY_LIMIT)) {
            logout();
            return;
        }

        // If valid, authorize and set/update activity
        setAuthorized(true);
        if (!lastActivity) {
            localStorage.setItem('lastActivity', now.toString());
        }

        // Setup activity trackers
        const updateActivity = () => {
            if (activeRef.current) return; // Debounce slightly if needed, or just set
            localStorage.setItem('lastActivity', Date.now().toString());
            activeRef.current = true;
            setTimeout(() => activeRef.current = false, 1000); // Throttle writes
        };

        // Ref for throttling
        const activeRef = { current: false };

        window.addEventListener('mousemove', updateActivity);
        window.addEventListener('keydown', updateActivity);
        window.addEventListener('click', updateActivity);

        // Interval to check periodically
        const interval = setInterval(() => {
            const last = localStorage.getItem('lastActivity');
            if (last && (Date.now() - parseInt(last, 10) > INACTIVITY_LIMIT)) {
                logout();
            }
        }, 60000); // Check every minute

        return () => {
            window.removeEventListener('mousemove', updateActivity);
            window.removeEventListener('keydown', updateActivity);
            window.removeEventListener('click', updateActivity);
            clearInterval(interval);
        };
    }, [pathname, router, logout]);

    // Prevent flash of protected content
    if (!authorized && pathname !== '/login') {
        return (
            <div className="min-h-screen bg-gray-950 flex items-center justify-center">
                <div className="text-blue-500 animate-pulse">Loading...</div>
            </div>
        );
    }

    return <>{children}</>;
}
