import torch
import torch.nn.functional as F
from torch.utils.data import Dataset, DataLoader
import random
from tqdm import tqdm
from .model import ModelConfig, Transformer

class NewtonSchulzLowRankDecay:
    def __init__(self, named_parameters, decay_rate=1e-3, num_iterations=5, target_keywords=None):
        self.decay_rate = decay_rate
        self.num_iterations = num_iterations
        self.target_keywords = target_keywords
        self.params_to_decay = []
        
        for name, param in named_parameters:
            if not param.requires_grad or param.ndim != 2:
                continue
            if self.target_keywords and not any(k in name for k in self.target_keywords):
                continue
            self.params_to_decay.append(param)
        
    @torch.no_grad()
    def step(self):
        for W in self.params_to_decay:
            orig_dtype = W.dtype
            X = W.float()
            r, c = X.shape
            
            transposed = False
            if r > c:
                X = X.T
                transposed = True
              
            norm = X.norm() + 1e-8
            X = X / norm
            
            Y = X
            I = torch.eye(min(r, c), device=X.device)
            
            for _ in range(self.num_iterations):
                A = Y.T @ Y
                Y = 0.5 * Y @ (3.0 * I - A)
            
            if transposed:
                Y = Y.T
            
            W.sub_(self.decay_rate * Y.to(orig_dtype))

class ModularAdditionDataset(Dataset):
    def __init__(self, p=113, split='train', train_frac=0.5, seed=42):
        data = [(i, j, p, (i + j) % p) for i in range(p) for j in range(p)]
        random.seed(seed)
        random.shuffle(data)
        split_idx = int(len(data) * train_frac)
        self.data = data[:split_idx] if split == 'train' else data[split_idx:]
    def __len__(self): return len(self.data)
    def __getitem__(self, idx):
        i, j, eq, res = self.data[idx]
        return torch.tensor([i, j, eq], dtype=torch.long), torch.tensor(res, dtype=torch.long)

def train_run(steps, train_frac, decay_type, decay_val, device):
    p = 113
    config = ModelConfig(vocab_size=p+1, use_qk_norm=True)
    model = Transformer(config).to(device)
    
    if decay_type == 'L2':
        # Standard L2 on everything
        optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=decay_val)
        lrd_opt = None
    else:
        decay_params, nodecay_params = [], []
        target = ["q_proj", "k_proj"]
        for name, p_val in model.named_parameters():
            if any(t in name for t in target): nodecay_params.append(p_val)
            else: decay_params.append(p_val)
            
        optimizer = torch.optim.AdamW([
            {'params': decay_params, 'weight_decay': 0.1}, # Keep mild L2 for stability
            {'params': nodecay_params, 'weight_decay': 0.0}
        ], lr=1e-3)
        lrd_opt = NewtonSchulzLowRankDecay(model.named_parameters(), decay_rate=decay_val, target_keywords=target)

    # Data
    train_ds = ModularAdditionDataset(p=p, split='train', train_frac=train_frac)
    val_ds = ModularAdditionDataset(p=p, split='val', train_frac=train_frac)
    train_loader = DataLoader(train_ds, batch_size=512, shuffle=True)
    val_loader = DataLoader(val_ds, batch_size=1024)
    
    # Loop
    max_val_acc = 0.0
    history = {'step': [], 'val_acc': [], 'rank': []}
    
    consecutive_high_acc = 0
    
    pbar = tqdm(range(steps), desc=f"Train({decay_type}={decay_val}, Frac={train_frac})", leave=False)
    iter_loader = iter(train_loader)
    
    for step in pbar:
        try: x, y = next(iter_loader)
        except: iter_loader = iter(train_loader); x, y = next(iter_loader)
        x, y = x.to(device), y.to(device)
        
        logits = model(x)
        loss = F.cross_entropy(logits, y)
        
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        if lrd_opt: lrd_opt.step()
        
        if step % 200 == 0:
            model.eval()
            corr, tot = 0, 0
            with torch.no_grad():
                for vx, vy in val_loader:
                    vx, vy = vx.to(device), vy.to(device)
                    corr += (model(vx).argmax(-1) == vy).sum().item()
                    tot += vy.size(0)
            val_acc = corr / tot
            model.train()
            
            # Simplified rank calc
            rank = 0.0 # Placeholder
            max_val_acc = max(max_val_acc, val_acc)
            
            history['step'].append(step)
            history['val_acc'].append(val_acc)
            history['rank'].append(rank)
            pbar.set_postfix({'acc': f"{val_acc:.2f}", 'rank': f"{rank:.2f}"})
            
            if val_acc > 0.99:
                consecutive_high_acc += 1
                if consecutive_high_acc >= 2:
                    break
            else:
                consecutive_high_acc = 0
                
    return max_val_acc, history, model
