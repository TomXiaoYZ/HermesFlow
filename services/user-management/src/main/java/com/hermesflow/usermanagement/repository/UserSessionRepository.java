package com.hermesflow.usermanagement.repository;

import com.hermesflow.usermanagement.entity.User;
import com.hermesflow.usermanagement.entity.UserSession;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * 用户会话数据访问接口
 * 提供用户会话相关的数据库操作方法
 */
@Repository
public interface UserSessionRepository extends JpaRepository<UserSession, UUID> {

    /**
     * 根据会话令牌查找会话
     * @param sessionToken 会话令牌
     * @return 会话信息
     */
    Optional<UserSession> findBySessionToken(String sessionToken);

    /**
     * 根据会话令牌和活跃状态查找会话
     * @param sessionToken 会话令牌
     * @param isActive 是否活跃
     * @return 会话信息
     */
    Optional<UserSession> findBySessionTokenAndIsActive(String sessionToken, Boolean isActive);

    /**
     * 根据刷新令牌查找会话
     * @param refreshToken 刷新令牌
     * @return 会话信息
     */
    Optional<UserSession> findByRefreshToken(String refreshToken);

    /**
     * 根据用户和活跃状态查找会话
     * @param user 用户
     * @param isActive 是否活跃
     * @return 会话列表
     */
    List<UserSession> findByUserAndIsActive(User user, Boolean isActive);

    /**
     * 查找用户的活跃会话
     * @param user 用户
     * @return 活跃会话列表
     */
    @Query("SELECT us FROM UserSession us WHERE us.user = :user AND us.isActive = true")
    List<UserSession> findActiveSessionsByUser(@Param("user") User user);

    /**
     * 查找已过期的会话
     * @return 过期会话列表
     */
    @Query("SELECT us FROM UserSession us WHERE us.isActive = true AND us.expiresAt < :now")
    List<UserSession> findExpiredSessions(@Param("now") LocalDateTime now);

    /**
     * 分页查询用户的会话
     * @param user 用户
     * @param pageable 分页参数
     * @return 分页会话列表
     */
    Page<UserSession> findByUser(User user, Pageable pageable);

    /**
     * 根据用户和活跃状态分页查询会话
     * @param user 用户
     * @param isActive 是否活跃
     * @param pageable 分页参数
     * @return 分页会话列表
     */
    Page<UserSession> findByUserAndIsActive(User user, Boolean isActive, Pageable pageable);

    /**
     * 根据用户代理查找用户会话
     * @param user 用户
     * @param userAgent 用户代理
     * @return 会话列表
     */
    List<UserSession> findByUserAndUserAgent(User user, String userAgent);

    /**
     * 查找指定时间之后活跃的会话
     * @param lastActivityAfter 最后活跃时间
     * @return 会话列表
     */
    List<UserSession> findByLastActivityAtAfter(LocalDateTime lastActivityAfter);

    /**
     * 查找长时间未活跃的会话
     * @param lastActivityBefore 最后活跃时间阈值
     * @return 会话列表
     */
    @Query("SELECT us FROM UserSession us WHERE us.isActive = true AND us.lastActivityAt < :lastActivityBefore")
    List<UserSession> findInactiveSessions(@Param("lastActivityBefore") LocalDateTime lastActivityBefore);

    /**
     * 统计用户的活跃会话数量
     * @param user 用户
     * @return 活跃会话数量
     */
    @Query("SELECT COUNT(us) FROM UserSession us WHERE us.user = :user AND us.isActive = true")
    long countActiveSessionsByUser(@Param("user") User user);

    /**
     * 统计活跃会话数量
     * @return 活跃会话数量
     */
    @Query("SELECT COUNT(us) FROM UserSession us WHERE us.isActive = true")
    long countActiveSessions();

    /**
     * 检查用户是否有活跃会话
     * @param user 用户
     * @return 是否有活跃会话
     */
    @Query("SELECT COUNT(us) > 0 FROM UserSession us WHERE us.user = :user AND us.isActive = true")
    boolean hasActiveSession(@Param("user") User user);

    /**
     * 批量更新过期会话状态
     * @param now 当前时间
     * @return 更新的记录数
     */
    @Modifying
    @Query("UPDATE UserSession us SET us.isActive = false WHERE us.isActive = true AND us.expiresAt < :now")
    int markExpiredSessions(@Param("now") LocalDateTime now);

    /**
     * 终止用户的所有活跃会话
     * @param user 用户
     * @return 更新的记录数
     */
    @Modifying
    @Query("UPDATE UserSession us SET us.isActive = false WHERE us.user = :user AND us.isActive = true")
    int terminateAllUserSessions(@Param("user") User user);

    /**
     * 根据用户ID终止用户的所有活跃会话
     * @param userId 用户ID
     * @return 更新的记录数
     */
    @Modifying
    @Query("UPDATE UserSession us SET us.isActive = false WHERE us.user.id = :userId AND us.isActive = true")
    int terminateAllUserSessions(@Param("userId") UUID userId);

    /**
     * 终止指定用户代理的会话
     * @param user 用户
     * @param userAgent 用户代理
     * @return 更新的记录数
     */
    @Modifying
    @Query("UPDATE UserSession us SET us.isActive = false WHERE us.user = :user AND us.userAgent = :userAgent AND us.isActive = true")
    int terminateUserAgentSessions(@Param("user") User user, @Param("userAgent") String userAgent);

    /**
     * 删除指定时间之前的非活跃会话
     * @param inactiveBefore 非活跃时间阈值
     */
    @Modifying
    @Query("DELETE FROM UserSession us WHERE us.isActive = false AND us.lastActivityAt < :inactiveBefore")
    void deleteOldInactiveSessions(@Param("inactiveBefore") LocalDateTime inactiveBefore);

    /**
     * 删除过期的会话
     * @param now 当前时间
     * @return 删除的记录数
     */
    @Modifying
    @Query("DELETE FROM UserSession us WHERE us.expiresAt < :now")
    int deleteExpiredSessions(@Param("now") LocalDateTime now);

    /**
     * 删除旧的已终止会话
     * @param threshold 时间阈值
     * @return 删除的记录数
     */
    @Modifying
    @Query("DELETE FROM UserSession us WHERE us.isActive = false AND us.lastActivityAt < :threshold")
    int deleteOldTerminatedSessions(@Param("threshold") LocalDateTime threshold);

    /**
     * 查找用户在指定时间范围内的会话
     * @param user 用户
     * @param startTime 开始时间
     * @param endTime 结束时间
     * @return 会话列表
     */
    @Query("SELECT us FROM UserSession us WHERE us.user = :user AND us.createdAt BETWEEN :startTime AND :endTime ORDER BY us.createdAt DESC")
    List<UserSession> findUserSessionsInTimeRange(@Param("user") User user, 
                                                  @Param("startTime") LocalDateTime startTime, 
                                                  @Param("endTime") LocalDateTime endTime);

    /**
     * 查找可疑的会话（同一用户多个活跃会话）
     * @param user 用户
     * @return 可疑会话列表
     */
    @Query("SELECT us FROM UserSession us WHERE us.user = :user AND us.isActive = true")
    List<UserSession> findActiveSessionsByUserForSecurity(@Param("user") User user);

    /**
     * 根据会话令牌检查会话是否存在且活跃
     * @param sessionToken 会话令牌
     * @return 是否存在且活跃
     */
    @Query("SELECT COUNT(us) > 0 FROM UserSession us WHERE us.sessionToken = :sessionToken AND us.isActive = true AND us.expiresAt > :now")
    boolean isSessionValid(@Param("sessionToken") String sessionToken, @Param("now") LocalDateTime now);
} 