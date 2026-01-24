import torch
import torch.nn as nn
import torch.nn.functional as F
from dataclasses import dataclass

@dataclass
class ModelConfig:
    vocab_size: int = 114
    dim: int = 128
    depth: int = 2
    heads: int = 4
    mlp_dim: int = 512
    use_qk_norm: bool = True 

class RMSNorm(nn.Module):
    def __init__(self, dim):
        super().__init__()
        self.scale = dim ** 0.5
        self.g = nn.Parameter(torch.ones(dim))
    def forward(self, x):
        return F.normalize(x, dim=-1) * self.scale * self.g

class Attention(nn.Module):
    def __init__(self, config):
        super().__init__()
        self.num_heads = config.heads
        self.head_dim = config.dim // config.heads
        self.scale = self.head_dim ** -0.5
        
        self.q_proj = nn.Linear(config.dim, config.dim, bias=False)
        self.k_proj = nn.Linear(config.dim, config.dim, bias=False)
        self.v_proj = nn.Linear(config.dim, config.dim, bias=False)
        self.o_proj = nn.Linear(config.dim, config.dim, bias=False)
        
        self.use_qk_norm = config.use_qk_norm
        if self.use_qk_norm:
            self.q_norm = RMSNorm(config.dim)
            self.k_norm = RMSNorm(config.dim)

    def forward(self, x):
        B, T, C = x.shape
        q, k, v = self.q_proj(x), self.k_proj(x), self.v_proj(x)
        
        if self.use_qk_norm:
            q, k = self.q_norm(q), self.k_norm(k)
            
        q = q.view(B, T, self.num_heads, self.head_dim).transpose(1, 2)
        k = k.view(B, T, self.num_heads, self.head_dim).transpose(1, 2)
        v = v.view(B, T, self.num_heads, self.head_dim).transpose(1, 2)
        
        attn = (q @ k.transpose(-2, -1)) * self.scale
        attn = attn.softmax(dim=-1)
        return self.o_proj((attn @ v).transpose(1, 2).reshape(B, T, C))

class Transformer(nn.Module):
    def __init__(self, config):
        super().__init__()
        self.config = config
        self.embedding = nn.Embedding(config.vocab_size, config.dim)
        self.pos_embedding = nn.Parameter(torch.randn(1, 3, config.dim) * 0.02)
        self.layers = nn.ModuleList([
            nn.ModuleDict({
                'norm1': RMSNorm(config.dim),
                'attn': Attention(config),
                'norm2': RMSNorm(config.dim),
                'mlp': nn.Sequential(
                    nn.Linear(config.dim, config.mlp_dim, bias=False),
                    nn.SiLU(),
                    nn.Linear(config.mlp_dim, config.dim, bias=False)
                )
            }) for _ in range(config.depth)
        ])
        self.norm_final = RMSNorm(config.dim)
        self.lm_head = nn.Linear(config.dim, config.vocab_size, bias=False)

    def forward(self, x):
        B, T = x.shape
        x = self.embedding(x) + self.pos_embedding[:, :T, :]
        for layer in self.layers:
            x = x + layer['attn'](layer['norm1'](x))
            x = x + layer['mlp'](layer['norm2'](x))
        x = self.norm_final(x)
        return self.lm_head(x[:, -1, :])
