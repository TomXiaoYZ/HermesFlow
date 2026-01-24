import click
import torch
from .train import train_run

@click.group()
def cli():
    pass

@cli.command()
@click.option('--steps', default=4000, help="Training steps")
@click.option('--device', default='cuda' if torch.cuda.is_available() else 'cpu', help="Device to use")
@click.option('--mode', default='mechanism', help="Experiment mode")
def train(steps, device, mode):
    print(f"Starting training in {mode} mode on {device}...")
    
    if mode == 'mechanism':
        # Single run mechanism analysis
        acc, _, _ = train_run(steps, 0.5, 'LowRank', 0.005, device)
        print(f"Training Complete. Final Acc: {acc:.4f}")
    else:
        print("Phase diagram mode not fully ported yet.")

if __name__ == "__main__":
    cli()
