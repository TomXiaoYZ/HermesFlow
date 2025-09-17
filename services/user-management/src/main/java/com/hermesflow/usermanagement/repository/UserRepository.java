package com.hermesflow.usermanagement.repository;

import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.entity.User;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * 用户数据访问接口
 * 提供用户相关的数据库操作方法，支持多租户隔离
 */
@Repository
public interface UserRepository extends JpaRepository<User, UUID> {

    /**
     * 根据用户名和租户查找用户
     * @param username 用户名
     * @param tenant 租户
     * @return 用户信息
     */
    Optional<User> findByUsernameAndTenant(String username, Tenant tenant);

    /**
     * 根据用户名查找用户（跨租户）
     * @param username 用户名
     * @return 用户信息
     */
    Optional<User> findByUsername(String username);

    /**
     * 根据邮箱和租户查找用户
     * @param email 邮箱
     * @param tenant 租户
     * @return 用户信息
     */
    Optional<User> findByEmailAndTenant(String email, Tenant tenant);

    /**
     * 查找租户下的活跃用户
     * @param tenant 租户
     * @return 活跃用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant = :tenant AND u.status = 'ACTIVE'")
    List<User> findActiveUsersByTenant(@Param("tenant") Tenant tenant);

    /**
     * 分页查询租户下的用户
     * @param tenant 租户
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    Page<User> findByTenant(Tenant tenant, Pageable pageable);

    /**
     * 根据状态和租户查询用户
     * @param tenant 租户
     * @param status 用户状态
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    Page<User> findByTenantAndStatus(Tenant tenant, User.UserStatus status, Pageable pageable);

    /**
     * 根据用户名模糊查询租户下的用户
     * @param tenant 租户
     * @param username 用户名关键字
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant = :tenant AND u.username LIKE %:username% ORDER BY u.createdAt DESC")
    Page<User> findByTenantAndUsernameContaining(@Param("tenant") Tenant tenant, 
                                                @Param("username") String username, 
                                                Pageable pageable);

    /**
     * 根据邮箱模糊查询租户下的用户
     * @param tenant 租户
     * @param email 邮箱关键字
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant = :tenant AND u.email LIKE %:email% ORDER BY u.createdAt DESC")
    Page<User> findByTenantAndEmailContaining(@Param("tenant") Tenant tenant, 
                                             @Param("email") String email, 
                                             Pageable pageable);

    /**
     * 统计租户下指定状态的用户数量
     * @param tenant 租户
     * @param status 用户状态
     * @return 用户数量
     */
    long countByTenantAndStatus(Tenant tenant, User.UserStatus status);

    /**
     * 统计租户下的用户总数
     * @param tenant 租户
     * @return 用户数量
     */
    long countByTenant(Tenant tenant);

    /**
     * 检查用户名在租户下是否存在
     * @param username 用户名
     * @param tenant 租户
     * @return 是否存在
     */
    boolean existsByUsernameAndTenant(String username, Tenant tenant);

    /**
     * 检查邮箱在租户下是否存在
     * @param email 邮箱
     * @param tenant 租户
     * @return 是否存在
     */
    boolean existsByEmailAndTenant(String email, Tenant tenant);

    /**
     * 查找指定时间之后登录的用户
     * @param tenant 租户
     * @param lastLoginAfter 最后登录时间
     * @return 用户列表
     */
    List<User> findByTenantAndLastLoginAtAfter(Tenant tenant, LocalDateTime lastLoginAfter);

    /**
     * 查找长时间未登录的用户
     * @param tenant 租户
     * @param lastLoginBefore 最后登录时间阈值
     * @return 用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant = :tenant AND (u.lastLoginAt IS NULL OR u.lastLoginAt < :lastLoginBefore)")
    List<User> findInactiveUsers(@Param("tenant") Tenant tenant, 
                                @Param("lastLoginBefore") LocalDateTime lastLoginBefore);

    /**
     * 查找需要密码重置的用户
     * @param tenant 租户
     * @param passwordChangedBefore 密码修改时间阈值
     * @return 用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant = :tenant AND u.passwordChangedAt < :passwordChangedBefore")
    List<User> findUsersRequiringPasswordReset(@Param("tenant") Tenant tenant, 
                                              @Param("passwordChangedBefore") LocalDateTime passwordChangedBefore);

    /**
     * 查找租户下的管理员用户
     * @param tenant 租户
     * @return 管理员用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant = :tenant AND u.status = 'ACTIVE'")
    List<User> findAdminUsersByTenant(@Param("tenant") Tenant tenant);

    // ========== 基于租户ID字符串的查询方法 ==========
    
    /**
     * 根据用户名和租户ID查找用户
     * @param username 用户名
     * @param tenantId 租户ID
     * @return 用户信息
     */
    @Query("SELECT u FROM User u WHERE u.username = :username AND u.tenant.id = :tenantId")
    Optional<User> findByUsernameAndTenantId(@Param("username") String username, @Param("tenantId") UUID tenantId);

    /**
     * 根据邮箱和租户ID查找用户
     * @param email 邮箱
     * @param tenantId 租户ID
     * @return 用户信息
     */
    @Query("SELECT u FROM User u WHERE u.email = :email AND u.tenant.id = :tenantId")
    Optional<User> findByEmailAndTenantId(@Param("email") String email, @Param("tenantId") UUID tenantId);

    /**
     * 根据用户ID和租户ID查找用户
     * @param id 用户ID
     * @param tenantId 租户ID
     * @return 用户信息
     */
    @Query("SELECT u FROM User u WHERE u.id = :id AND u.tenant.id = :tenantId")
    Optional<User> findByIdAndTenantId(@Param("id") UUID id, @Param("tenantId") UUID tenantId);

    /**
     * 分页查询租户下的用户（基于租户ID）
     * @param tenantId 租户ID
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant.id = :tenantId ORDER BY u.createdAt DESC")
    Page<User> findByTenantId(@Param("tenantId") UUID tenantId, Pageable pageable);

    /**
     * 检查用户名在租户下是否存在（基于租户ID）
     * @param username 用户名
     * @param tenantId 租户ID
     * @return 是否存在
     */
    @Query("SELECT COUNT(u) > 0 FROM User u WHERE u.username = :username AND u.tenant.id = :tenantId")
    boolean existsByUsernameAndTenantId(@Param("username") String username, @Param("tenantId") UUID tenantId);

    /**
     * 检查邮箱在租户下是否存在（基于租户ID）
     * @param email 邮箱
     * @param tenantId 租户ID
     * @return 是否存在
     */
    @Query("SELECT COUNT(u) > 0 FROM User u WHERE u.email = :email AND u.tenant.id = :tenantId")
    boolean existsByEmailAndTenantId(@Param("email") String email, @Param("tenantId") UUID tenantId);

    /**
     * 统计租户下的用户总数（基于租户ID）
     * @param tenantId 租户ID
     * @return 用户数量
     */
    @Query("SELECT COUNT(u) FROM User u WHERE u.tenant.id = :tenantId")
    long countByTenantId(@Param("tenantId") UUID tenantId);

    /**
     * 查找租户下的活跃用户（基于租户ID）
     * @param tenantId 租户ID
     * @return 活跃用户列表
     */
    @Query("SELECT u FROM User u WHERE u.tenant.id = :tenantId AND u.status = 'ACTIVE'")
    List<User> findActiveUsersByTenantId(@Param("tenantId") UUID tenantId);
}