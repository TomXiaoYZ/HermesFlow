"use client";

import ExchangeConfig from "@/components/Settings/ExchangeConfig";

export default function SettingsPage() {
    return (
        <div className="min-h-screen bg-[#0B0E14] text-white p-6 md:p-8">
            <div className="max-w-7xl mx-auto">
                <ExchangeConfig />
            </div>
        </div>
    );
}
