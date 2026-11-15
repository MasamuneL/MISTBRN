# Evidencia P2 preparación offline — 2026-11-13 America/Mexico_City

- RED: `pwsh -NoProfile -File tools/test-world-preparation.ps1`, exit 1: helper inexistente, provisión requerida sin implementación.
- GREEN: mismo comando, exit 0; fixtures conservados en `target/world-preparation-tests/2020b1ccb9b04c49b7db3ed660d95937`. Cubre provisión, idempotencia, recuperación en ambas direcciones, JSON malformado/futuro/duplicado/extra/tipos/ausencias/ID, conflicto, UTF-8 inválido, tamaño, temporal parcial, directorio no vacío y contención estricta.
- Parser PowerShell AST ambos scripts: cero errores, exit 0.
- SymbolicLink y Junction: creación denegada por el entorno; sus rechazos no se pudieron ejecutar. No se simuló un fallo IO real. No se ejecutó juego ni mundo personal.
- SHA256 `prepare-world.ps1`: `D2925907B8E92F0F17E9F5D5D9584083C1721978180FDD9F24E424E8F342993A`; `test-world-preparation.ps1`: `5420D5E6F5BD39E4A8615BF6E1004BA4951D2DD9FA56FE278AC34599652AAA48`.
- `Flush(true)` no prueba resistencia a pérdida de energía. Comprobación de procesos y rutas requiere operación exclusiva; no es bloqueo contra cambios concurrentes. La protección de producto corresponde al guard nativo y sus gates pendientes.
