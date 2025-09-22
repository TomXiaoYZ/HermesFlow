package com.hermesflow.permissionmanagement.exception;

/**
 * 资源未找到异常类
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
public class ResourceNotFoundException extends PermissionManagementException {
    
    public ResourceNotFoundException(String message) {
        super(message, "RESOURCE_NOT_FOUND");
    }
    
    public ResourceNotFoundException(String resourceType, String resourceId) {
        super(String.format("%s with id %s not found", resourceType, resourceId), "RESOURCE_NOT_FOUND");
    }
    
    public ResourceNotFoundException(String message, Throwable cause) {
        super(message, "RESOURCE_NOT_FOUND", cause);
    }
} 