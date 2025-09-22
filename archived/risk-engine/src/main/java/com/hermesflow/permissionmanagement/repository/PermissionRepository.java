package com.hermesflow.permissionmanagement.repository;

import com.hermesflow.permissionmanagement.entity.Permission;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

/**
 * 权限Repository接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Repository
public interface PermissionRepository extends JpaRepository<Permission, UUID> {

    /**
     * 根据权限代码查找权限
     */
    Optional<Permission> findByCode(String code);

    /**
     * 根据权限代码列表查找权限
     */
    List<Permission> findByCodeIn(Set<String> codes);

    /**
     * 根据资源类型查找权限
     */
    List<Permission> findByResource(String resource);

    /**
     * 根据资源类型和操作查找权限
     */
    Optional<Permission> findByResourceAndAction(String resource, String action);

    /**
     * 根据权限类型查找权限
     */
    List<Permission> findByPermissionType(Permission.PermissionType permissionType);

    /**
     * 查找系统权限
     */
    List<Permission> findByIsSystemTrue();

    /**
     * 查找非系统权限
     */
    List<Permission> findByIsSystemFalse();

    /**
     * 根据资源类型分页查询权限
     */
    Page<Permission> findByResource(String resource, Pageable pageable);

    /**
     * 根据权限类型分页查询权限
     */
    Page<Permission> findByPermissionType(Permission.PermissionType permissionType, Pageable pageable);

    /**
     * 模糊查询权限 (根据名称或代码)
     */
    @Query("SELECT p FROM Permission p WHERE " +
           "LOWER(p.name) LIKE LOWER(CONCAT('%', :keyword, '%')) OR " +
           "LOWER(p.code) LIKE LOWER(CONCAT('%', :keyword, '%')) OR " +
           "LOWER(p.description) LIKE LOWER(CONCAT('%', :keyword, '%'))")
    Page<Permission> searchByKeyword(@Param("keyword") String keyword, Pageable pageable);

    /**
     * 根据资源类型列表查找权限
     */
    List<Permission> findByResourceIn(Set<String> resources);

    /**
     * 检查权限代码是否存在
     */
    boolean existsByCode(String code);

    /**
     * 检查资源和操作组合是否存在
     */
    boolean existsByResourceAndAction(String resource, String action);

    /**
     * 统计系统权限数量
     */
    long countByIsSystemTrue();

    /**
     * 根据权限类型统计数量
     */
    long countByPermissionType(Permission.PermissionType permissionType);
} 