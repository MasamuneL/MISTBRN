# Compatibilidad de Alomancia con Horizon

P0 tiene ensayos mínimos ejecutados y revisión independiente final documentada; el cierre C0 lo registra el checkpoint principal. Aún no hay mod jugable ni entrega de las ocho variantes.

## Revisión y herramientas fijadas

- Windows x64; instalación base `nightly-2026-11-08-fdc87ae`.
- Commit exacto: `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`; fuente descargada por launcher y `.horizon-revision` cotejada. `NATIVE_HOOKS_API = 411` en esa revisión; el probe usa pin completo, no compatibilidad supuesta con una API mínima.
- Rust portable1.99.1 (`x86_64-pc-windows-gnu`) del launcher, instalado solo en scratch. El manifiesto fijado exige Rust>=1.95/edition2024. No depender del resumen antiguo1.90+ del README.
- `.hnmod` es un bundle Rust enlazado al juego por el launcher; se probó el bundle de autoría `horizon-mixin.toml` + `[api]`. `.hmod` es contenido declarativo; se cargó un directorio de autoría con `horizon-mod.toml` mediante `--dev-mod`. Empaquetado final de ambos sigue en P10.
- Recursos de la instalación local reutilizados mediante copia aislada. No se leyó cuenta ni se modificaron mundos personales; scratch y backups no se distribuyen.

La revisión main `36eb98b6f6ec4f22649cbe65bbfd32bc0dc2083f` (API415) sirvió como comparación estática, no como versión objetivo. Los helpers de fragments también entran en generación: no inferir ausencia de una API solo porque no esté en el catálogo raíz.

## Matriz de capacidad del candidato fijado

Todas las pruebas usaron el servidor integrado y un mundo desechable respaldado. Los JSON `p0-runtime-v2/v4/v5.json` contienen resultados curados. La [matriz pública del pin API411](execution/evidence/p0-pin-api.md) fija las firmas Hooks/Host, generados y cuerpos exactos consumidos por el probe. [p0-api.md](execution/evidence/p0-api.md) queda como comparación estática de main/API415; no documenta las firmas del target. [p0-persistence-api.md](execution/evidence/p0-persistence-api.md) amplía la revisión de persistencia del pin.

| Capacidad | Ensayo ejecutado | Estado / límites |
|---|---|---|
| Carga nativa | Cinco builds seriales; factories/hooks ejecutados en juego | Soportada en pin. Fuente final v5 compiló exit0. |
| Entrada | Z down/repeat/up, inventario y chat mediante CLI oficial | Dos ensayos positivos; una activación sostenida, ninguna extra en menús/chat. Alt-tab físico y reasignación por UI no probados. |
| Tick | Marcadores limitados de cliente y servidor integrados | Ejecutado; orden/pausa/freeze y simulación del mod se verificarán P1/P2/P4. |
| Item y contenido | Fixture original registrada:1659 items vs1658 base; give/select/use1muestra, callback use_item | Ejecutado. Receta declarada y JSON válidos, fabricación y consumo transaccional no probados (P3). |
| Impulso/colisión | Impulso vertical0.30: y79→79.296; ensayo techo sólido, dos posiciones muestreadas y79 sin penetración | Movimiento normal del motor observado; no máxima trayectoria, velocidad total, anclajes ni proyectiles (P4). |
| HUD | Rectángulo cyan/negro propio capturado e inspeccionado a1280×720 | Ejecutado. Pantallas completas y escalas de UI esperan P7. Captura solo local. |
| Combate/loot | Pig generado en scratch, /kill produjo living_death y loot_drops(count1) | Hooks ejecutados. No prueba atribución de participación/jefes ni hitos personales (P6). |
| Guardado | Attachment contador entre procesos; además NeoForgeData versionado0→1 tras cerrar/reabrir | Helpers inline ejecutados en v5; elegirlos como estado canónico. Cortes/rollback/replay todavía P2. |
| Sonido | play_sound_to, categoría player: retorno sound_queued=true; v3 con players devolviófalse y corrigió v4 | Encolado ejecutado; export sin audio, audición y recursos propios no probados. |
| Servidor/mensajes | Ping servidor→cliente y ACK cliente→servidor, canal/payload fijo validado | Servidor integrado probado. No dos clientes, servidor dedicado ni resistencia a ataques/latencia (P9). |

## Decisión de integración aprobada en C0

Continuar modo individual con núcleo Rust independiente y adaptador nativo sobre ese commit, sin fork. Economía/progresión y eventos verificados viven en el servidor integrado; el cliente solicita comandos y muestra efectos. Un impulso cliente no acredita autoridad de movimiento para cooperativo: no entregar compatibilidad vanilla ni cooperativo por estos resultados.

Guardar el estado canónico en un subcompound versionado de `NeoForgeData` usando `Host::player_persistent_data`/`set_player_persistent_data`, preservando namespaces ajenos. Está inline en el mismo snapshot Guest que Inventory. P2 debe aplicar inventario exacto y progreso juntos dentro de callback servidor, validar antes y deshacer ambos en memoria ante fallo. Los helpers de slots rechazan jugador swapped-in: encolar la intención al contexto servidor apropiado, no asumir que funcionan desde use_item.

El getter `player_persistent_data` convierte un `NeoForgeData` presente de tipo inválido en compound vacío (`native_player_state.rs:70–75`). Por ello el guard `root_type` del probe no detecta corrupción de raíz on-disk; el round-trip observado solo acredita datos válidos. **Contrato obligatorio P2:** fuera del turno del jugador, leer `Host::player_compound` y validar el tag completo antes de cualquier escritura o consumo. Distinguir raíz ausente (inicialización permitida) de raíz presente no-Compound (rechazar y conservar sin sobrescribir); validar también namespace, esquema futuro y campos propios, preservando raíz y namespaces ajenos. Dentro del turno `player_compound` devuelve None: encolar la intención y comprobar en el contexto servidor apropiado. No basar esa protección únicamente en el getter normalizado.

No elegir Attachments como economía canónica del dueño: la revisión de fuente detectó autosave que serializa inventario antes de refrescar ese mapa separado. El contador de cierre normal no demuestra seguridad del autosave. Attachments pueden servir para réplica de UI después de validación.

El escritor normal reemplaza `.dat` con backup `.dat_old`, pero no hace fsync por acción; ni ACK ni SaveToFile son recibo durable de una transacción. P2 debe probar snapshots coherentes viejo/nuevo, huecos de renombre, replay, rollback y errores. No se promete resistencia a pérdida de energía ni transacción entre jugador y chunk/jefe. Si los ensayos P2 no satisfacen los requisitos, volver a esta decisión antes de ampliar; no introducir un fork en silencio.

## Reproducción y evidencia

Receta completa del build v5, conservada para reproducir un **export nuevo**. No es una orden de repetir un build ya verificado: en esta sesión `probe-engine-v5` ya existe; cotejar artefacto, hash y logs primero. El launcher observado está en `%LOCALAPPDATA%\Programs\Horizon Launcher\horizon-launcher.exe` del usuario normal. Se resuelve abajo desde `$env:USERPROFILE` (ruta comprobada), porque el sandbox puede redirigir `$env:LOCALAPPDATA` a otra carpeta. Ejecutar la receta desde la raíz del workspace.

**Prerrequisito ya realizado:** la versión legítima y preparada está copiada en `$launcherScratch/versions/nightly-2026-11-08-fdc87ae`, incluidos ejecutables/marcadores y `.local/horizon-client/26.3`. Esa copia aislada se hizo desde la instalación legítima; no contiene mundos ni cuentas. La receta comprueba su presencia, no repite ni inventa preparación de recursos. No redistribuir esa carpeta.

```powershell
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath '.').Path
$scratch = Join-Path $projectRoot 'experiments/horizon-probe/runtime'
$launcherScratch = Join-Path $scratch 'launcher'
$launcher = Join-Path $env:USERPROFILE 'AppData/Local/Programs/Horizon Launcher/horizon-launcher.exe'
$bundle = (Resolve-Path -LiteralPath (Join-Path $projectRoot 'experiments/horizon-probe/bundle')).Path
$export = Join-Path $scratch 'probe-engine-v5'
$preparedBase = Join-Path $launcherScratch 'versions/nightly-2026-11-08-fdc87ae/.local/horizon-client/26.3'
if (!(Test-Path -LiteralPath $launcher)) { throw 'No se encuentra el launcher observado.' }
if (!(Test-Path -LiteralPath $preparedBase)) { throw 'Falta la base legítima ya preparada en launcher scratch; revisar el inventario y logs de copia.' }
if (Test-Path -LiteralPath $export) { throw 'El export v5 ya existe: inspeccionarlo; no sobrescribir ni repetir el build.' }
$env:HORIZON_HOME = Join-Path $scratch 'home-v2'
$env:HORIZON_LAUNCHER_HOME = $launcherScratch
$env:XDG_DATA_HOME = Join-Path $scratch 'data'
$env:XDG_CONFIG_HOME = Join-Path $scratch 'config'
$env:HORIZON_LAUNCHER_TRASH = Join-Path $scratch 'trash'
$env:HORIZON_LAUNCHER_CLIPBOARD = Join-Path $scratch 'clipboard'
$env:HORIZON_LAUNCHER_DESKTOP = Join-Path $scratch 'desktop'
$env:HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
$env:HORIZON_NO_AUDIO = '1'
Push-Location -LiteralPath $projectRoot
try {
    & $launcher native-build $bundle --version nightly-2026-11-08-fdc87ae --trust-native-code --profile dev --server-output $export
    if ($LASTEXITCODE -ne 0) { throw "native-build terminó con exit $LASTEXITCODE" }
} finally {
    Pop-Location
}
```

`experiments/horizon-probe/smoke.ps1` ejecuta solo el export aislado e incluye endpoints únicos por RunId, captura de exitcode y timeout. Ejemplo tras construir el export v5:

```powershell
./experiments/horizon-probe/smoke.ps1 -RunId storage-check -EngineVersion probe-engine-v5 -Content -TimeoutSeconds 180
```

Pasar `-Controls` requiere acceso al socket local; el sandbox produjo error10013. El primer arranque con contenido reconstruyó tablas y tardó103.8s, por eso se amplió el timeout a180s con esperas acotadas. Un endpoint de una ejecución anterior produjo10061; se corrigió usando uno por RunId, sin cambiar firewall.

Los logs crudos, screenshots, binarios, mundos y recursos permanecen locales bajo rutas ignoradas. No reinstalar toolchain ni repetir builds verificados al recuperar una sesión: cotejar SHA, logs, procesos y `CHECKPOINT.md` primero.