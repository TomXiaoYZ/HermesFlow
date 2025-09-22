import React from 'react';
import { ArrowUpIcon, ArrowDownIcon, BellIcon, ChartBarIcon, CurrencyDollarIcon, UserCircleIcon } from '@heroicons/react/24/solid';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const data = [
  { name: '00:00', BTC: 45000 },
  { name: '04:00', BTC: 46200 },
  { name: '08:00', BTC: 45800 },
  { name: '12:00', BTC: 47100 },
  { name: '16:00', BTC: 46800 },
  { name: '20:00', BTC: 47500 },
  { name: '24:00', BTC: 48000 },
];

const stats = [
  {
    name: '总市值',
    value: '￥2.5万亿',
    change: '3.2%',
    isIncrease: true,
    icon: CurrencyDollarIcon,
  },
  {
    name: '24h交易量',
    value: '￥1200亿',
    change: '2.1%',
    isIncrease: false,
    icon: ChartBarIcon,
  },
  {
    name: 'BTC占比',
    value: '42.5%',
    change: '0.5%',
    isIncrease: true,
    icon: ChartBarIcon,
  },
  {
    name: '交易所数量',
    value: '500+',
    change: '',
    isIncrease: true,
    icon: ChartBarIcon,
  }
];

const topCryptos = [
  { name: 'Bitcoin', symbol: 'BTC', price: '￥320,150.00', change: '+2.5%', volume: '￥1,234亿' },
  { name: 'Ethereum', symbol: 'ETH', price: '￥18,750.00', change: '-1.2%', volume: '￥567亿' },
  { name: 'Binance Coin', symbol: 'BNB', price: '￥2,150.00', change: '+0.8%', volume: '￥123亿' },
];

function Dashboard() {
  return (
    <div className="min-h-screen bg-gray-50">
      {/* Navigation */}
      <nav className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <h1 className="text-2xl font-bold text-indigo-600">HermesFlow</h1>
            </div>
            <div className="flex items-center space-x-4">
              <button className="p-2 rounded-full text-gray-400 hover:text-gray-500">
                <BellIcon className="h-6 w-6" />
              </button>
              <div className="relative">
                <img
                  className="h-8 w-8 rounded-full"
                  src="https://images.pexels.com/photos/2379005/pexels-photo-2379005.jpeg"
                  alt="User avatar"
                />
              </div>
            </div>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        {/* Stats Grid */}
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4 mb-8">
          {stats.map((item) => (
            <div key={item.name} className="bg-white overflow-hidden shadow rounded-lg">
              <div className="px-4 py-5 sm:p-6">
                <div className="flex items-center">
                  <div className="flex-shrink-0">
                    <item.icon className="h-6 w-6 text-gray-400" />
                  </div>
                  <div className="ml-5 w-0 flex-1">
                    <dt className="text-sm font-medium text-gray-500 truncate">{item.name}</dt>
                    <dd className="flex items-baseline">
                      <div className="text-2xl font-semibold text-gray-900">{item.value}</div>
                      {item.change && (
                        <div className={`ml-2 flex items-baseline text-sm font-semibold ${item.isIncrease ? 'text-green-600' : 'text-red-600'}`}>
                          {item.isIncrease ? <ArrowUpIcon className="h-4 w-4" /> : <ArrowDownIcon className="h-4 w-4" />}
                          <span className="sr-only">{item.isIncrease ? '增加' : '减少'}</span>
                          {item.change}
                        </div>
                      )}
                    </dd>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Chart and Market Data */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Price Chart */}
          <div className="lg:col-span-2 bg-white rounded-lg shadow p-6">
            <h2 className="text-lg font-medium text-gray-900 mb-4">BTC/USDT</h2>
            <div className="h-80">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={data}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="name" />
                  <YAxis />
                  <Tooltip />
                  <Line type="monotone" dataKey="BTC" stroke="#4F46E5" strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>

          {/* Market Overview */}
          <div className="bg-white rounded-lg shadow">
            <div className="p-6">
              <h2 className="text-lg font-medium text-gray-900 mb-4">市场概览</h2>
              <div className="space-y-4">
                {topCryptos.map((crypto) => (
                  <div key={crypto.symbol} className="flex items-center justify-between">
                    <div>
                      <div className="text-sm font-medium text-gray-900">{crypto.name}</div>
                      <div className="text-sm text-gray-500">{crypto.symbol}</div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm font-medium text-gray-900">{crypto.price}</div>
                      <div className={`text-sm ${crypto.change.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                        {crypto.change}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default Dashboard;