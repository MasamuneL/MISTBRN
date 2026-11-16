Continúa el cierre de los gates pendientes de P2 del mod de alomancia para Horizon. El diseño y el plan siguen aprobados; no reinicies ni repitas trabajo válido. La petición de completar P2 sigue autorizada.

WORKTREE ACTIVO (mismo checkout):
C:\Users\alanj.VIKINGO\.codex\worktrees\mistbrn-p1\MISTBRN
Rama: codex/alomancia-p2-gates
HEAD al relevo: 8bbdf7397fecf9ddc8f92d63d66c396bf198fa21
Repositorio: https://github.com/MasamuneL/MISTBRN.git
Runtime/SDK/toolchain compartidos:
C:\Users\alanj.VIKINGO\OneDrive\Documents\ChatGPT\MISTBRN\experiments\horizon-probe\runtime

Lee primero desde el worktree:
1. docs/execution/CHECKPOINT.md schema16, especialmente el ÚLTIMO bloque RELEVO REAL 2026-11-16.
2. docs/execution/HANDOFF.md, cabecera 2026-11-16.
3. docs/execution/tasks/P2-native-completion.md y MAINTENANCE.md.
4. docs/superpowers/specs/2026-11-12-alomancia-design.md y plans/2026-11-12-alomancia.md.
5. docs/execution/evidence/p2-storage-architecture.md, N0–N9.
6. docs/execution/tasks/P2-native-economy-completion.md, P2-native-gate-fixtures.md y P2-native-writer-boundaries.md.
7. Evidencias p2-native-cuts.md/p2-cuts-checks.json y nuevos archivos gates-*/rich-*.

Antes de modificar/compilar/probar, comprueba Git status/rama/HEAD, procesos, archivos temporales, receipts y artefactos reales. Hay bastante trabajo gates SIN COMMIT, incluidos tres archivos Rust, helpers y evidencias. Presérvalo; no reset/clean/checkout destructivo, no otro worktree ni copia/reinstalación SDK. Root conserva CHECKPOINT/manifiestos/Cargo/builds/commits; un solo build o ensayo a la vez (el ensayo dedicado requiere su servidor y un cliente propios).

Preserva sin stage automático estos tres cambios ajenos:
- docs/execution/evidence/p0-initial-probe.rs
- docs/execution/evidence/p2-horizon-preflight.md
- experiments/horizon-probe/bundle/api/src/lib.rs

Estado real:
- P1 integrado en main; P2 aún parcial, sin poderes jugables ni merge P2 a main.
- HEAD8bbdf73 guarda N6: cuatro cortes reales del adaptador + cuatro reaperturas, revisados. 47 comprobaciones anteriores consolidadas. No repetirlas/core sin cambios relevantes o dudas justificadas.
- Build vigente p2-cuts1, engine SHA256 8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70. NO se compiló gates1. Fuentes actuales son posteriores a esa build.
- Diez nuevos receipts PASS pendientes de consolidación final: siete gates de identidad/restauración, segundo mundo de identidad nueva, provisión parcial con backend deshabilitado y rich-reconnect-3 con fracciones/desbloqueos/reglas/ledger no triviales. No declarar 57 como consolidado sin revisar receipts.
- FALLO conservado rich-dead-login-3: receipt failed/pass=false, Timeout, termination=left_running, PID29208. Log WorldCloseFinal luego frame-limited run timed out/crash report/Press Enter. Al relevo active_count=0 para Horizon/Cargo/rustc/gdb; no quedó ese PID vivo. Diagnosticar harness/pantalla de muerte/limitador de frames; NO contarlo PASS ni reutilizar ese intento. Rich death-respawn/dimension/save-pause todavía no ejecutados. Intentos rich1/2 sólo bloquearon en preflight por juego personal, sin ensayos; el usuario lo cerró.
- Helpers de fixtures y oráculo están implementados con selftests. Se corrigió un fallo del inspector: inventario NBT vacío legal lleva child0, no child10; RED en snapshot real y GREEN offline. No reabrir ese diagnóstico sin evidencia nueva.

Código nuevo aún no compilado:
- lib.rs/save.rs/native_test.rs: conflict, replay-new, capacity, full-ledger, stale-slot, stale-generation, reentrant, disconnected-intent; añadido DedicatedServer autoritativo y autosave-pending en world_saved(wait=false), token de armado y contador real de SaveToFile dentro del helper de inventario.
- Rustfmt --check exit0 al relevo, pero los ÚLTIMOS cambios N5 no tienen revisión final/typecheck/build/ejecución. Los hashes frozen antiguos de la ficha están superados; usar los hashes reales del último CHECKPOINT. Revisar esos cambios y completar lo que falte antes de una única build nueva.
- tools/test-native-dedicated.ps1 NO existe: Astra agotó cuota antes de entregarlo. No asumir preparado/ejecutado.
- tools/native-writer-debug.gdb SÍ existe, carga y resolución de símbolos estáticas comprobadas; ningún corte N7 con debugger se ha ejecutado. Está fijado a cuts1, no reutilizar offsets con otra build.

Autorización persistente: el usuario YA aceptó expresamente la EULA de Minecraft y autorizó eula=true SOLO en el directorio de ensayo y servidor limitado a127.0.0.1. No volver a pedirlo. Servidor dedicado aún no arrancado.

Próximas acciones:
1. Revisar estado/source N5 real y completar controladores dedicado/N7; diagnosticar el fallo dead-login sin repetir éxitos.
2. N5 FIFO requiere guest dedicado real: integrado owner guarda síncronamente y no tiene esa cola. Debe probarse callback realmente pendiente y drenado dentro del helper, no inferirlo de sleep ni fabricar hooks. Leer el plan concreto de GDB/SuspendThread/ResumeThread propio en el relevo/documentación; verificar PID y TID exactos y detener sólo procesos propios.
3. N7 necesita fronteras reales del writer. GDB incluido bajo launcher/rust/1.99.1-x86_64-pc-windows-gnu/mingw64/bin/gdb.exe. El helper captura NtWriteFile exitoso antes del cierre de fs::write y retornos de renombres; no prueba corte dentro del kernel/escritura parcial ni fsync. Guard estricto sigue rechazando principal ausente aunque old sea válido; restaurar únicamente mundo COMPLETO offline en otra copia.
4. Quit/kick del owner integrado NO acreditan logging_out real antes del siguiente tick. Usar salida genuina del guest y evidencia de orden. Terminar N3/N4/N5/N7/N8/N9 y auditar cobertura N1/N2 antes de declarar P2 completa.
5. Medir runtime antes de build, revisar fuentes, compilar una vez si cambia Rust, comprobar inputs/core único y ejecutar serialmente sobre clones cerrados; preservar todos los intentos, hashes/oráculos A/B y reapertura de otras copias. Revisión independiente antes de commit/integración; no push/merge main automático.

Usa subagentes según plan y propiedad exclusiva: Sol high integración/revisión, Terra medium helpers, Luna medium documentación, Astra high persistencia compleja. Comprueba disponibilidad una vez; Astra agotó cuota en este bloque, sin bucles de reintentos; sustitución Sol/root documentada si corresponde.

Mantén checkpoints antes y después de cambios/comandos/pruebas/resultados/fallos. Conserva cuts1 para N7 fijado, owner1 baseline, Native2 fallo histórico, P1/P0 necesarios, mundos/backups/evidencias/SDK/target/dependencias. No repitas limpiezas antiguas ni acumules exports por ensayo. Continúa autónomamente; consulta sólo por bloqueo real o cambio sustancial de alcance.
