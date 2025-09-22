package com.hermesflow.usermanagement.security;

import com.hermesflow.usermanagement.entity.User;
import org.springframework.security.core.GrantedAuthority;
import org.springframework.security.core.authority.SimpleGrantedAuthority;
import org.springframework.security.core.userdetails.UserDetails;

import java.util.Collection;
import java.util.List;
import java.util.Objects;

public class UserPrincipal implements UserDetails {
    private String id;
    private String username;
    private String email;
    private String password;
    private String tenantId;
    private Collection<? extends GrantedAuthority> authorities;

    public UserPrincipal(String id, String username, String email, String password, 
                        String tenantId, Collection<? extends GrantedAuthority> authorities) {
        this.id = id;
        this.username = username;
        this.email = email;
        this.password = password;
        this.tenantId = tenantId;
        this.authorities = authorities;
    }

    public static UserPrincipal create(User user) {
        List<GrantedAuthority> authorities = List.of(
            new SimpleGrantedAuthority("ROLE_USER")
        );

        return new UserPrincipal(
                user.getId().toString(),
                user.getUsername(),
                user.getEmail(),
                user.getPassword(),
                user.getTenant().getId().toString(),
                authorities
        );
    }

    public String getId() {
        return id;
    }

    public String getEmail() {
        return email;
    }

    public String getTenantId() {
        return tenantId;
    }

    @Override
    public String getUsername() {
        return username;
    }

    @Override
    public String getPassword() {
        return password;
    }

    @Override
    public Collection<? extends GrantedAuthority> getAuthorities() {
        return authorities;
    }

    @Override
    public boolean isAccountNonExpired() {
        return true;
    }

    @Override
    public boolean isAccountNonLocked() {
        return true;
    }

    @Override
    public boolean isCredentialsNonExpired() {
        return true;
    }

    @Override
    public boolean isEnabled() {
        return true;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        UserPrincipal that = (UserPrincipal) o;
        return Objects.equals(id, that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id);
    }
} 