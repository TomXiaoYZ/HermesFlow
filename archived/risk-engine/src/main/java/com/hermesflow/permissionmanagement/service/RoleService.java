package com.hermesflow.permissionmanagement.service;

import java.util.UUID;
import java.util.List;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import com.hermesflow.permissionmanagement.dto.*;

public interface RoleService {
    RoleDTO createRole(RoleCreateDTO createDTO);
    List<RoleDTO> batchCreateRoles(List<RoleCreateDTO> createDTOs);
    RoleDTO updateRole(UUID id, RoleUpdateDTO updateDTO);
    void deleteRole(UUID id);
    RoleDTO getRoleById(UUID id);
    RoleDTO getRoleByCode(String code, UUID tenantId);
    Page<RoleDTO> getRolesByTenantId(UUID tenantId, Pageable pageable);
    Page<RoleDTO> getRolesByType(String type, UUID tenantId, Pageable pageable);
    List<RoleDTO> getActiveRolesByTenantId(UUID tenantId);
    Page<RoleDTO> searchRoles(String keyword, UUID tenantId, Pageable pageable);
    List<RoleDTO> getChildRoles(UUID id);
    void activateRole(UUID id);
    void deactivateRole(UUID id);
    void assignPermissionToRole(UUID roleId, UUID permissionId, UUID grantedBy);
    void removePermissionFromRole(UUID roleId, UUID permissionId);
    boolean checkRolePermission(UUID roleId, String permissionCode);
    boolean validateRoleCode(String code, UUID tenantId);
    RoleUsageDTO getRoleUsage(UUID id);
    RoleStatistics getRoleStatistics(UUID tenantId);
    
    class RoleStatistics {
        private long totalRoles;
        public RoleStatistics() {}
        public long getTotalRoles() { return totalRoles; }
        public void setTotalRoles(long totalRoles) { this.totalRoles = totalRoles; }
    }
}