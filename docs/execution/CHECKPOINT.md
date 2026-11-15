# Checkpoint de ejecución — Alomancia Horizon
schema_version: 11
fecha: 2026-11-15 America/Mexico_City
estado: P0/P1 integrated; P2 native compiled, N0/N1 parciales; owner identity pendiente
rama: codex/alomancia-p2-native
HEAD_base: fed744b9252e14ae6fcd12b314fba61040250fe0
worktree: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN
original_runtime: C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime

## Estado real vigente
- P1 main24b4b2c; P2A reservas y P2B codec reviewed; P3 catálogo/validator reviewed interno. Sin poderes ni P2 completo; no merge de P2.
- Nueva rama p2-native preservó WIP original. Identity helper revisión Sol aprobada; backend cuarentena estáticamente revisado Astra, guard rechaza marca por presencia. Fuente core sin cambios tras evidencia71+doctest/fmt previa, no suite repetida.
- Native SDK build p2-native1 falló por borrow driver E0505; fix mínimo; p2-native2 succeeded exit0 real, engine SHA0A51CCBF7D1D695B0467E41842214B9F4E3ABCFF7CD418CE7AE97F391DB9E0BB. Fuente pin API411 sin edición, un único core via stage native-crates/allomancy/core, input hashes iguales. Native-test default OFF fuente, sólo stage opt-in.
- N0 acotado perfil offline Player: 17ensayos pass (2 aperturas, foreign aceptado y14rechazos). Incluye IO real FileShare.None, corrupción/futuro/duplicados/.DAT/backup/markers/cuarentena. Todos rechazos preadopción sin login, hashes intactos; salida controlada-1 explícita, no cierre limpio. Evidencia p2-native-n0.md/json y receipts native-open-runs. NO N0 general.
- roundtrip2 credit/replay inmediato Host real PASS, quit y WorldCloseFinal exit0; offline reserva1000/ledger1/iron_ingot2, UUID/payload iguales. p2-native-credit.md. Replay tras restart/cortes/lifecycle/rollback reales N1–N9 pendientes; ningún fsync inferido.
- Defecto SDK owner: primera apertura con perfil automático18eb guardó namespace18eb bajo alias a01e(offline Player), reopen PlayerMismatch correcto. Snapshot fallido inmutable y fuente diagnosticados en P2-native-identity-debug.md. No reparar datos, no alias/guard relajado. Fixtures derivadas fallidas previas NO acreditadas.

## Próximo corte y decisión
Ruling P2-native-owner-architecture.md: bundle firstclass congela Launch.player exacto a options del worker, HEAD valida perfil owner antes adopción, bootstrap fija Level.uuid sólo mundo marcado realmente nuevo antes primerwrite. Existentes mismatch se rechazan, sin migración; preserve vanilla sin marker. Root interpreta adaptación reversible como necesaria para identidad P2 ya autorizada, no fork/upstream silencioso. Revisión independiente Sol P2-native-owner-review.md en curso (Astra cuota, sustitución según plan). NO targets compilados aún.
Intención ANTES: guardar commit WIP explícito de native2/evidencias (no fases completas), sin stage de diferenciasP0 ni evidence/p2-horizon-preflight preexistente. Después crear rama nueva codex/alomancia-p2-owner en mismo worktree; revisar HEAD/status antes de continuar owner. Ningún otro checkout/SDKcopy. Implementadores por archivos exclusivos, principal manifests/descriptores/Cargo/build/commits/checkpoint, un Cargo a la vez.
Siguiente secuencia: confirmar WIPcommit+branch; esperar revisión contratoowner; congelar archivos/APIs; implementar helpers+descriptor mínimo; planner real selectores únicos; build nuevo nombre; mundo nuevo perfil arbitrario, reopen/save/autosave y mismatch existing hashes; revisión antes merge. N1–N9 no cerrar por compile/unit tests.

## Seguridad y mantenimiento
No procesos juego/build propios vivos al cierre de roundtrip2. Mantener runtime actual p2-native2-engine/cachefc09 y baseline P1bridge2/cache250494 + probev5 necesario P0; no exports duplicados creados por helper. Sólo una export P2 válida nueva; staging intentos fuente+receipts conservados. Sin limpieza de mundos/backups/target/SDK. Ya se recuperaron8.82GiB antes, no repetir limpieza antigua. Medir antes cada build grande.
Preparador fix frontera AllowedRoot inclusive + FileInfo.Directory probado suite0/junctionpass; symlink unavailable por Admin, no false pass. Logs sharing corregidos y receipts fallidos preservados. Historial completo de esta reanudación en history/CHECKPOINT-native2-2026-11-15.md y .bak. Fuente WIP original siempre preservada.
Toolchain/shared paths y reglas originales en HANDOFF (actualizar estado antes del siguiente cierre); pin fdc87ae/Rust1.99.1/API411. Read skill/process ya aplicado; no reiniciar diseño/plan.
