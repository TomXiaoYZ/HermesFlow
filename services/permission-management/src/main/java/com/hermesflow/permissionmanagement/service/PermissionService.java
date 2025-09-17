package com.hermesflow.permissionmanagement.service;

import com.hermesflow.permissionmanagement.dto.PermissionCreateDTO;
import com.hermesflow.permissionmanagement.dto.PermissionDTO;
import com.hermesflow.permissionmanagement.dto.PermissionUpdateDTO;
import com.hermesflow.permissionmanagement.entity.Permission;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;

import java.util.List;
import java.util.Set;
import java.util.UUID;

/**
 * 权限管理服务接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public interface PermissionService {

    /**
     * 创建权限
     * 
     * @param createDTO 权限创建DTO
     * @return 权限DTO
     */
    PermissionDTO createPermission(PermissionCreateDTO createDTO);

    /**
     * 更新权限
     * 
     * @param permissionId 权限ID
     * @param updateDTO 权限更新DTO
     * @return 权限DTO
     */
    PermissionDTO updatePermission(UUID permissionId, PermissionUpdateDTO updateDTO);

    /**
     * 删除权限
     * 
     * @param permissionId 权限ID
     */
    void deletePermission(UUID permissionId);

    /**
     * 根据ID获取权限
     * 
     * @param permissionId 权限ID
     * @return 权限DTO
     */
    PermissionDTO getPermissionById(UUID permissionId);

    /**
     * 根据权限代码获取权限
     * 
     * @param code 权限代码
     * @return 权限DTO
     */
    PermissionDTO getPermissionByCode(String code);

    /**
     * 根据权限代码批量获取权限
     * 
     * @param codes 权限代码集合
     * @return 权限DTO列表
     */
    List<PermissionDTO> getPermissionsByCodes(Set<String> codes);

    /**
     * 根据资源获取权限列表
     * 
     * @param resource 资源名称
     * @return 权限DTO列表
     */
    List<PermissionDTO> getPermissionsByResource(String resource);

    /**
     * 根据资源和操作获取权限
     * 
     * @param resource 资源名称
     * @param action 操作名称
     * @return 权限DTO
     */
    PermissionDTO getPermissionByResourceAndAction(String resource, String action);

    /**
     * 根据权限类型获取权限列表
     * 
     * @param permissionType 权限类型
     * @return 权限DTO列表
     */
    List<PermissionDTO> getPermissionsByType(Permission.PermissionType permissionType);

    /**
     * 分页查询所有权限
     * 
     * @param pageable 分页参数
     * @return 权限DTO分页结果
     */
    Page<PermissionDTO> getAllPermissions(Pageable pageable);

    /**
     * 根据关键词搜索权限
     * 
     * @param keyword 搜索关键词
     * @param pageable 分页参数
     * @return 权限DTO分页结果
     */
    Page<PermissionDTO> searchPermissions(String keyword, Pageable pageable);

    /**
     * 检查权限代码是否存在
     * 
     * @param code 权限代码
     * @return 是否存在
     */
    boolean existsByCode(String code);

    /**
     * 检查资源和操作组合是否存在
     * 
     * @param resource 资源名称
     * @param action 操作名称
     * @return 是否存在
     */
    boolean existsByResourceAndAction(String resource, String action);

    /**
     * 批量创建权限
     * 
     * @param createDTOs 权限创建DTO列表
     * @return 权限DTO列表
     */
    List<PermissionDTO> batchCreatePermissions(List<PermissionCreateDTO> createDTOs);

    /**
     * 批量删除权限
     * 
     * @param permissionIds 权限ID集合
     */
    void batchDeletePermissions(Set<UUID> permissionIds);

    /**
     * 分页查询所有权限
     * 
     * @param pageable 分页参数
     * @return 权限DTO分页结果
     */
    Page<PermissionDTO> getPermissions(Pageable pageable);

    /**
     * 根据权限类型分页查询权限
     * 
     * @param type 权限类型
     * @param pageable 分页参数
     * @return 权限DTO分页结果
     */
    Page<PermissionDTO> getPermissionsByType(String type, Pageable pageable);

    /**
     * 根据资源分页查询权限
     * 
     * @param resource 资源名称
     * @param pageable 分页参数
     * @return 权限DTO分页结果
     */
    Page<PermissionDTO> getPermissionsByResource(String resource, Pageable pageable);

    /**
     * 获取系统权限列表
     * 
     * @return 系统权限DTO列表
     */
    List<PermissionDTO> getSystemPermissions();

    /**
     * 激活权限
     * 
     * @param permissionId 权限ID
     */
    void activatePermission(UUID permissionId);

    /**
     * 停用权限
     * 
     * @param permissionId 权限ID
     */
    void deactivatePermission(UUID permissionId);

    /**
     * 初始化系统默认权限
     * 
     * @return 创建的权限数量
     */
    List<PermissionDTO> initializeDefaultPermissions();

    /**
     * 验证权限代码格式
     * 
     * @param code 权限代码
     * @return 是否有效
     */
    boolean validatePermissionCode(String code);

    /**
     * 获取权限统计信息
     * 
     * @return 权限统计信息
     */
    PermissionStatistics getPermissionStatistics();

    /**
     * 权限统计信息内部类
     */
    class PermissionStatistics {
        private long totalPermissions;
        private long systemPermissions;
        private long functionalPermissions;
        private long dataPermissions;
        private long systemTypePermissions;

        // 构造函数
        public PermissionStatistics(long totalPermissions, long systemPermissions, 
                                  long functionalPermissions, long dataPermissions, 
                                  long systemTypePermissions) {
            this.totalPermissions = totalPermissions;
            this.systemPermissions = systemPermissions;
            this.functionalPermissions = functionalPermissions;
            this.dataPermissions = dataPermissions;
            this.systemTypePermissions = systemTypePermissions;
        }

        // Getters
        public long getTotalPermissions() { return totalPermissions; }
        public long getSystemPermissions() { return systemPermissions; }
        public long getFunctionalPermissions() { return functionalPermissions; }
        public long getDataPermissions() { return dataPermissions; }
        public long getSystemTypePermissions() { return systemTypePermissions; }
    }
}