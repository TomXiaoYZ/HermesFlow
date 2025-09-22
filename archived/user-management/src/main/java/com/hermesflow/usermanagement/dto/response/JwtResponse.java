package com.hermesflow.usermanagement.dto.response;

public class JwtResponse {
    private String token;
    private String type;
    private Long expiresIn;

    public JwtResponse(String accessToken, String tokenType) {
        this.token = accessToken;
        this.type = tokenType;
        this.expiresIn = 86400L; // 24 hours in seconds
    }

    public JwtResponse(String accessToken, String tokenType, Long expiresIn) {
        this.token = accessToken;
        this.type = tokenType;
        this.expiresIn = expiresIn;
    }

    public String getToken() {
        return token;
    }

    public void setToken(String token) {
        this.token = token;
    }

    public String getType() {
        return type;
    }

    public void setType(String type) {
        this.type = type;
    }

    public Long getExpiresIn() {
        return expiresIn;
    }

    public void setExpiresIn(Long expiresIn) {
        this.expiresIn = expiresIn;
    }
} 