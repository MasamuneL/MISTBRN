# Masamune's MISTBRN

Mod de alomancia para [Horizon](https://playhorizon.dev), inspirado en Mistborn: metales, reservas, poderes por parejas y progresión por práctica e investigación. Id del mod: `mistbrn`. En desarrollo; todavía no hay versión jugable.

**Multiplataforma:** el mod funciona en todo sistema que soporte el SDK fijado de Horizon (macOS arm64, Windows x64, Linux x64). El desarrollo y las pruebas en juego se hacen primero en macOS arm64.

## Estado

Fijado a Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb…`, juego 26.3, Rust 1.99.1). El estado vigente y la siguiente acción están en [`docs/STATUS.md`](docs/STATUS.md); el plan en [`docs/plan.md`](docs/plan.md); el diseño en [`docs/design.md`](docs/design.md). El trabajo anterior (Windows, `fdc87ae`) está congelado en la rama `legacy/windows-fdc87ae` ([resumen](docs/history.md)).

## Estructura

```text
mod/                  paquete del mod (.hmod plano)
  horizon-mod.toml    identidad y pin de Horizon
  src/*.rs            adaptador Horizon
  src/core/           crate allomancy-core: reglas puras, sin dependencias, probado con cargo
tools/                scripts bash portables (macOS, Linux, Git Bash en Windows)
docs/                 plan, diseño, contratos, compatibilidad, evidencias
```

## Desarrollo

```sh
tools/fetch-sdk.sh            # SDK fijado en mod/.horizon/sdk (ignorado por git)
tools/check.sh                # formato, clippy, pruebas del núcleo y control de peso del repo
tools/build-native.sh NAME    # build nativo con el launcher para la instancia mistbrn-dev
```

Las pruebas en juego usan solo la instancia `mistbrn-dev` y mundos `mistbrn-*`. Runtime, logs, SDK y binarios quedan fuera del repo.

## Distribución

No incluye Minecraft, recursos del juego, binarios ni material de las novelas.
