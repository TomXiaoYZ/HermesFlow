'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';

export default function AuthGuard({ children }: { children: React.ReactNode }) {
    const router = useRouter();
    const pathname = usePathname();
    const [authorized, setAuthorized] = useState(false);

    useEffect(() => {
        // Allow public access to login
        if (pathname === '/login') {
            setAuthorized(true);
            return;
        }

        const token = localStorage.getItem('token');
        if (!token) {
            // Redirect to login
            router.push('/login');
        } else {
            setAuthorized(true);
        }
    }, [pathname, router]);

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
