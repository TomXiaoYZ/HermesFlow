import React, { useState } from 'react';

// Mock Data
const INITIAL_USERS = [
  { id: 1, name: 'Alex Chen', email: 'alex.c@hermesflow.com', role: 'Admin', status: 'Active', lastLogin: 'Just now', access: 'Full Access' },
  { id: 2, name: 'Sarah Wu', email: 'sarah.w@hermesflow.com', role: 'Senior Trader', status: 'Active', lastLogin: '2h ago', access: 'Trading + L2 Data' },
  { id: 3, name: 'Mike Johnson', email: 'mike.j@hermesflow.com', role: 'Quant Analyst', status: 'Active', lastLogin: '5h ago', access: 'Research Only' },
  { id: 4, name: 'Guest Viewer', email: 'guest@partner.com', role: 'Viewer', status: 'Suspended', lastLogin: '3d ago', access: 'Read Only' },
];

const Card: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`bg-surface-900 border border-surface-800 rounded-md overflow-hidden ${className}`}>
    {children}
  </div>
);

export const UserManagement: React.FC = () => {
  const [users, setUsers] = useState(INITIAL_USERS);
  const [searchTerm, setSearchTerm] = useState('');

  const filteredUsers = users.filter(u => 
    u.name.toLowerCase().includes(searchTerm.toLowerCase()) || 
    u.email.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="h-full overflow-y-auto p-6 flex flex-col gap-6 bg-surface-950">
      {/* Header */}
      <div className="flex justify-between items-end">
        <div>
           <h2 className="text-2xl font-bold text-surface-100 mb-1 flex items-center gap-2">
              <span className="material-symbols-outlined text-hermes-500">admin_panel_settings</span>
              用户权限管理 (User Management)
           </h2>
           <p className="text-xs text-surface-500">管理团队成员访问权限与系统角色</p>
        </div>
        <button className="px-4 py-2 bg-hermes-500 hover:bg-hermes-400 text-surface-950 font-bold text-sm rounded shadow-lg shadow-hermes-500/20 transition-colors flex items-center gap-2">
            <span className="material-symbols-outlined text-[18px]">person_add</span>
            新增用户
        </button>
      </div>

      {/* Stats Row */}
      <div className="grid grid-cols-4 gap-4">
         <Card className="p-4 flex items-center gap-4">
            <div className="w-10 h-10 rounded-full bg-surface-800 flex items-center justify-center text-surface-400">
               <span className="material-symbols-outlined">group</span>
            </div>
            <div>
               <div className="text-2xl font-bold text-surface-100">{users.length}</div>
               <div className="text-xs text-surface-500 uppercase font-bold">总用户数</div>
            </div>
         </Card>
         <Card className="p-4 flex items-center gap-4">
            <div className="w-10 h-10 rounded-full bg-hermes-500/10 flex items-center justify-center text-hermes-500">
               <span className="material-symbols-outlined">how_to_reg</span>
            </div>
            <div>
               <div className="text-2xl font-bold text-surface-100">{users.filter(u => u.status === 'Active').length}</div>
               <div className="text-xs text-surface-500 uppercase font-bold">活跃账户</div>
            </div>
         </Card>
         <Card className="p-4 flex items-center gap-4">
            <div className="w-10 h-10 rounded-full bg-purple-500/10 flex items-center justify-center text-purple-500">
               <span className="material-symbols-outlined">shield_person</span>
            </div>
            <div>
               <div className="text-2xl font-bold text-surface-100">{users.filter(u => u.role === 'Admin').length}</div>
               <div className="text-xs text-surface-500 uppercase font-bold">管理员</div>
            </div>
         </Card>
         <Card className="p-4 flex items-center gap-4 border-trade-warn/30 bg-trade-warn/5">
            <div className="w-10 h-10 rounded-full bg-trade-warn/10 flex items-center justify-center text-trade-warn">
               <span className="material-symbols-outlined">lock_clock</span>
            </div>
            <div>
               <div className="text-2xl font-bold text-trade-warn">{users.filter(u => u.status === 'Suspended').length}</div>
               <div className="text-xs text-trade-warn/70 uppercase font-bold">已冻结/暂停</div>
            </div>
         </Card>
      </div>

      {/* Main Table Card */}
      <Card className="flex-1 flex flex-col min-h-0">
         {/* Toolbar */}
         <div className="p-4 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
            <div className="relative w-64">
               <span className="absolute left-3 top-2.5 material-symbols-outlined text-surface-500 text-[18px]">search</span>
               <input 
                  type="text" 
                  placeholder="搜索姓名或邮箱..." 
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="w-full bg-surface-950 border border-surface-700 rounded-md py-2 pl-9 pr-3 text-sm text-surface-200 outline-none focus:border-surface-500" 
               />
            </div>
            <div className="flex gap-2">
               <button className="px-3 py-1.5 border border-surface-700 rounded text-xs text-surface-400 hover:text-surface-200 flex items-center gap-1">
                  <span className="material-symbols-outlined text-[16px]">filter_list</span> 筛选
               </button>
               <button className="px-3 py-1.5 border border-surface-700 rounded text-xs text-surface-400 hover:text-surface-200 flex items-center gap-1">
                  <span className="material-symbols-outlined text-[16px]">download</span> 导出列表
               </button>
            </div>
         </div>

         {/* Table */}
         <div className="flex-1 overflow-auto">
            <table className="w-full text-left text-sm">
               <thead className="bg-surface-950 text-surface-500 font-bold text-xs uppercase sticky top-0 z-10">
                  <tr>
                     <th className="px-6 py-4 border-b border-surface-800">用户 (User)</th>
                     <th className="px-6 py-4 border-b border-surface-800">角色 (Role)</th>
                     <th className="px-6 py-4 border-b border-surface-800">状态 (Status)</th>
                     <th className="px-6 py-4 border-b border-surface-800">数据权限 (Access)</th>
                     <th className="px-6 py-4 border-b border-surface-800">最后活跃 (Last Active)</th>
                     <th className="px-6 py-4 border-b border-surface-800 text-right">操作</th>
                  </tr>
               </thead>
               <tbody className="divide-y divide-surface-800">
                  {filteredUsers.map(user => (
                     <tr key={user.id} className="hover:bg-surface-800/50 transition-colors group">
                        <td className="px-6 py-4">
                           <div className="flex items-center gap-3">
                              <div className="w-9 h-9 rounded-full bg-surface-800 flex items-center justify-center text-xs font-bold text-surface-300 border border-surface-700">
                                 {user.name.split(' ').map(n => n[0]).join('')}
                              </div>
                              <div>
                                 <div className="font-bold text-surface-200">{user.name}</div>
                                 <div className="text-xs text-surface-500">{user.email}</div>
                              </div>
                           </div>
                        </td>
                        <td className="px-6 py-4">
                           <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border ${
                              user.role === 'Admin' ? 'bg-purple-500/10 text-purple-400 border-purple-500/20' :
                              user.role.includes('Trader') ? 'bg-hermes-500/10 text-hermes-500 border-hermes-500/20' :
                              'bg-surface-800 text-surface-400 border-surface-700'
                           }`}>
                              {user.role === 'Admin' && <span className="material-symbols-outlined text-[14px]">verified_user</span>}
                              {user.role}
                           </span>
                        </td>
                        <td className="px-6 py-4">
                           <div className="flex items-center gap-2">
                              <div className={`w-2 h-2 rounded-full ${user.status === 'Active' ? 'bg-trade-up shadow-[0_0_5px_#00E396]' : 'bg-trade-down'}`}></div>
                              <span className={`text-xs font-medium ${user.status === 'Active' ? 'text-surface-300' : 'text-surface-500'}`}>{user.status}</span>
                           </div>
                        </td>
                        <td className="px-6 py-4">
                           <span className="text-xs font-mono text-surface-400 bg-surface-900 px-2 py-1 rounded border border-surface-800">
                              {user.access}
                           </span>
                        </td>
                        <td className="px-6 py-4 text-surface-400 text-xs">
                           {user.lastLogin}
                        </td>
                        <td className="px-6 py-4 text-right">
                           <div className="flex justify-end gap-1 opacity-60 group-hover:opacity-100 transition-opacity">
                              <button className="p-1.5 hover:bg-surface-700 rounded text-surface-400 hover:text-white transition-colors" title="编辑">
                                 <span className="material-symbols-outlined text-[18px]">edit</span>
                              </button>
                              <button className="p-1.5 hover:bg-surface-700 rounded text-surface-400 hover:text-white transition-colors" title="重置密码">
                                 <span className="material-symbols-outlined text-[18px]">lock_reset</span>
                              </button>
                              <button className="p-1.5 hover:bg-trade-down/20 rounded text-surface-400 hover:text-trade-down transition-colors" title="禁用/删除">
                                 <span className="material-symbols-outlined text-[18px]">block</span>
                              </button>
                           </div>
                        </td>
                     </tr>
                  ))}
               </tbody>
            </table>
         </div>
         
         {/* Pagination Footer */}
         <div className="p-3 border-t border-surface-800 bg-surface-950/30 flex justify-between items-center text-xs text-surface-500">
            <div>显示 1-{filteredUsers.length} 共 {filteredUsers.length} 条记录</div>
            <div className="flex gap-2">
               <button className="px-3 py-1 bg-surface-900 border border-surface-800 rounded hover:bg-surface-800 disabled:opacity-50">上一页</button>
               <button className="px-3 py-1 bg-surface-900 border border-surface-800 rounded hover:bg-surface-800 disabled:opacity-50">下一页</button>
            </div>
         </div>
      </Card>
    </div>
  );
};