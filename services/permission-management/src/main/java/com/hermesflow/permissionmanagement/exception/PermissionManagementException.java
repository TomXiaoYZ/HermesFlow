package com.hermesflow.permissionmanagement.exception;

/**
 * 权限管理异常类
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
public class PermissionManagementException extends RuntimeException {
    
    private final String errorCode;
    
    public PermissionManagementException(String message) {
        super(message);
        this.errorCode = "PERMISSION_ERROR";
    }
    
    public PermissionManagementException(String message, String errorCode) {
        super(message);
        this.errorCode = errorCode;
    }
    
    public PermissionManagementException(String message, Throwable cause) {
        super(message, cause);
        this.errorCode = "PERMISSION_ERROR";
    }
    
    public PermissionManagementException(String message, String errorCode, Throwable cause) {
        super(message, cause);
        this.errorCode = errorCode;
    }
    
    public String getErrorCode() {
        return errorCode;
    }
} 