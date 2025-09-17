package com.hermesflow.permissionmanagement.exception;

/**
 * 业务异常类
 * 用于处理业务逻辑中的异常情况
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
public class BusinessException extends RuntimeException {
    
    private String errorCode;
    
    public BusinessException(String message) {
        super(message);
    }
    
    public BusinessException(String errorCode, String message) {
        super(message);
        this.errorCode = errorCode;
    }
    
    public BusinessException(String message, Throwable cause) {
        super(message, cause);
    }
    
    public BusinessException(String errorCode, String message, Throwable cause) {
        super(message, cause);
        this.errorCode = errorCode;
    }
    
    public String getErrorCode() {
        return errorCode;
    }
    
    public void setErrorCode(String errorCode) {
        this.errorCode = errorCode;
    }
} 