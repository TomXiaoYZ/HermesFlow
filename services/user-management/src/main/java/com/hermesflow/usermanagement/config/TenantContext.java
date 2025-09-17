package com.hermesflow.usermanagement.config;

import org.springframework.stereotype.Component;
import java.util.UUID;

/**
 * 租户上下文管理器
 * 
 * 用于在当前线程中管理租户信息，支持多租户数据隔离。
 * 通过ThreadLocal确保线程安全，每个请求线程都有独立的租户上下文。
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-12-19
 */
@Component
public class TenantContext {
    
    /**
     * 使用ThreadLocal存储当前线程的租户ID
     * 确保多线程环境下的租户隔离
     */
    private static final ThreadLocal<UUID> currentTenant = new ThreadLocal<>();
    
    /**
     * 设置当前线程的租户ID
     * 
     * @param tenantId 租户ID，不能为null
     * @throws IllegalArgumentException 如果tenantId为null
     */
    public void setCurrentTenantId(UUID tenantId) {
        if (tenantId == null) {
            throw new IllegalArgumentException("Tenant ID cannot be null");
        }
        currentTenant.set(tenantId);
    }
    
    /**
     * 获取当前线程的租户ID
     * 
     * @return 当前租户ID，如果未设置则返回null
     */
    public UUID getCurrentTenantId() {
        return currentTenant.get();
    }
    
    /**
     * 检查当前线程是否设置了租户上下文
     * 
     * @return 如果已设置租户上下文返回true，否则返回false
     */
    public boolean hasTenantContext() {
        return currentTenant.get() != null;
    }
    
    /**
     * 清除当前线程的租户上下文
     * 
     * 重要：必须在请求结束时调用此方法，避免内存泄漏
     * 通常在拦截器的afterCompletion方法中调用
     */
    public void clear() {
        currentTenant.remove();
    }
    
    /**
     * 获取当前租户ID的字符串表示
     * 
     * @return 租户ID字符串，如果未设置则返回null
     */
    public String getCurrentTenantIdAsString() {
        UUID tenantId = getCurrentTenantId();
        return tenantId != null ? tenantId.toString() : null;
    }
    
    /**
     * 在指定租户上下文中执行操作
     * 
     * 这是一个工具方法，用于临时切换租户上下文执行特定操作，
     * 操作完成后自动恢复原有的租户上下文。
     * 
     * @param tenantId 临时租户ID
     * @param operation 要执行的操作
     * @param <T> 操作返回值类型
     * @return 操作执行结果
     * @throws RuntimeException 如果操作执行过程中发生异常
     */
    public <T> T executeInTenantContext(UUID tenantId, TenantOperation<T> operation) {
        UUID originalTenantId = getCurrentTenantId();
        try {
            setCurrentTenantId(tenantId);
            return operation.execute();
        } catch (Exception e) {
            // 将检查异常包装为运行时异常
            throw new RuntimeException("执行租户上下文操作时发生异常", e);
        } finally {
            if (originalTenantId != null) {
                setCurrentTenantId(originalTenantId);
            } else {
                clear();
            }
        }
    }
    
    /**
     * 租户操作函数式接口
     * 
     * @param <T> 操作返回值类型
     */
    @FunctionalInterface
    public interface TenantOperation<T> {
        /**
         * 执行操作
         * 
         * @return 操作结果
         * @throws Exception 操作过程中可能抛出的异常
         */
        T execute() throws Exception;
    }
    
    /**
     * 获取当前租户上下文的调试信息
     * 
     * @return 包含租户信息的调试字符串
     */
    @Override
    public String toString() {
        UUID tenantId = getCurrentTenantId();
        return String.format("TenantContext{tenantId=%s, thread=%s}", 
                           tenantId, Thread.currentThread().getName());
    }
} 