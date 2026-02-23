# web - FOLDER_INDEX

> Dashboard UI built with Next.js 16, React 19, and Tailwind CSS 4. Server-side rendered with `output: "standalone"` for Docker deployment.

## Module Map

```
src/
  app/
    layout.tsx                 # Root layout (metadata, fonts, global styles)
    page.tsx                   # Home page → Dashboard component
    login/page.tsx             # Login page (auth form)
    market/overview/page.tsx   # Market overview page
    settings/page.tsx          # Settings page

  components/
    AuthGuard.tsx              # Auth state wrapper (redirect to login if unauthenticated)
    Dashboard.tsx              # Main dashboard: tab-based navigation between views
    MarketOverview.tsx         # Real-time market data display
    DataPipeline.tsx           # Data pipeline status + collector health
    DataDiscovery.tsx          # Data exploration (quality + SQL IDE + table browser)
    StrategyLab.tsx            # Strategy evolution monitoring
    EvolutionExplorer.tsx      # GA evolution visualization (fitness curves, genomes)
    LiveTrading.tsx            # Live trading status + positions
    TradeExecutionPanel.tsx    # Manual trade execution panel
    SystemStatus.tsx           # Service health dashboard
    SystemLogs.tsx             # ClickHouse log viewer

    data-discovery/
      QualityDashboard.tsx     # Data quality metrics display
      SqlIde.tsx               # SQL query editor (ClickHouse)
      TableBrowser.tsx         # Database table browser

    Settings/
      ExchangeConfig.tsx       # Exchange configuration management
      TradingAccountConfig.tsx # Trading account settings

  lib/
    utils.ts                   # Utility functions (cn, formatters)

  utils/
    genome.ts                  # Genome decoder (RPN → human-readable formula)
```

## API Proxy

All `/api/*` requests are rewritten to `gateway:8080` at build time via `next.config.mjs`:
```
/api/:path* → http://gateway:8080/api/:path*
```

WebSocket: `NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws` (client-side, connects directly)

## Dependencies
- Next.js 16, React 19, Tailwind CSS 4
- Recharts (charting)
- Radix UI primitives
