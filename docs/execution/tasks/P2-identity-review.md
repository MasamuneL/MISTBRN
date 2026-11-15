# P2 identity — revisión independiente estática

Veredicto: **APROBADO para el helper de identidad**, sin hallazgos accionables en el alcance revisado. Esto no aprueba P2 completa ni el guard nativo.

## Base y alcance

- Worktree: `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN`.
- Rama comprobada: `codex/alomancia-p2-native`.
- HEAD comprobado: `fed744b9252e14ae6fcd12b314fba61040250fe0`; revisión del WIP presente, no solo del commit.
- Autoridad: `docs/execution/tasks/P2-native.md:12–15` para marcador/API y `:24` para propiedad.
- Lectura completa del contrato, documentación, módulo y suite de identidad; diff de `lib.rs` contra HEAD. Consulta auxiliar de `types.rs` para verificar construcción e igualdad de `WorldId`.
- No se modificó código, no se ejecutaron Cargo, builds, tests ni fmt y no se crearon commits.

## Hallazgos y cumplimiento

**Ningún defecto accionable encontrado.** Las referencias siguientes describen las comprobaciones estáticas, no resultados de pruebas ejecutadas por esta revisión.

| Contrato | Evidencia revisada |
| --- | --- |
| UTF-8 estricto y límite 4096 bytes | `src/world_identity.rs:64–76` limita antes del parseo y valida UTF-8; `:101–111` lee como máximo 4097 bytes. Tests `tests/world_identity.rs:109–125`. |
| Objeto exacto, campos únicos y tipos v1 | Visitor de mapa en `src/world_identity.rs:40–62`, comparación de nombres ya decodificados; versión y forma en `:77–90`. Rechaza arrays/null, extras y duplicados también con nombres escapados. Tests `:71–101`. |
| Versiones futuras y UUID N minúsculo | `src/world_identity.rs:77–98` informa Future antes de interpretar campos v1, exige 32 bytes hex ASCII minúsculos y no normaliza ni genera identidad. Tests `:44–69` y `:102–107`. |
| Política principal/backup | `src/world_identity.rs:119–137`: ambos se validan; principal inválido termina con error; igualdad semántica por WorldId; conflicto rechaza; ausencia doble da None; backup único devuelve fuente Backup. Tests `:127–209`. |
| IO y ausencia de mutación | `src/world_identity.rs:101–111` solo convierte NotFound en ausencia; demás errores open/read devuelven Io. El módulo solo abre/lee, no crea, escribe ni repara. Tests `:154–166`, `:180–202` y `:211–223`. |
| API y propiedad | Nombres/tipos públicos coinciden con P2-native; diff `src/lib.rs:13–14` agrega únicamente módulo y reexport. Usa WorldId existente, sin dependencias del juego ni codec alternativo. |
| Documentación del helper | `docs/world-identity.md:5–34` describe la forma, API, precedencia y recuperación explícita observadas. La distinción entre tests del núcleo y gates nativos se conserva en `:58–60`. |

Rutas `src/…` y `tests/…` de la tabla relativas a `crates/allomancy-core/`.

## Hashes de los archivos revisados

SHA-256 de bytes del WIP, incluidos finales de línea:

| Archivo | SHA-256 |
| --- | --- |
| `docs/execution/tasks/P2-native.md` | `DA93097BBA1D00C8F730ED03F6B270FD077EB245A5AD40B5745A5B015640BB92` |
| `docs/world-identity.md` | `5B64F0CEC5C44734505C8263B638055B7DDDD1FF9BABEE8FC7C7E9BFCE7D62AB` |
| `crates/allomancy-core/src/world_identity.rs` | `A0B1E9ABBCB9A26F528C723ACD578653AA014D6D676B1CFC1DE90DB213422B1A` |
| `crates/allomancy-core/tests/world_identity.rs` | `04010345B62114274157847686DB76631ACD6028C240A5DA6C8A928F74886E5C` |
| `crates/allomancy-core/src/lib.rs` | `028148DC1C80FBFD30689649C8DB27A29DABCAD7445051B73A58D4FD8525DE73` |
| `crates/allomancy-core/src/types.rs` (consulta auxiliar) | `88A855816A280A11779326112F6AF21DF7F01EE12281DF2167926D64FA08564E` |

## Límites y evidencia recibida

La coordinación reporta suite de identidad 16, workspace 71, doctest y fmt aprobados en este mismo código. No se repitieron ni se auditaron sus logs; el veredicto aquí es independiente y estático. Los tests existentes expresan los comportamientos anteriores, pero no demuestran por sí solos todos los fallos posibles de permisos/lectura o cambios concurrentes de archivos.

No se revisaron `preflight.rs`, `world_guard.rs`, backend, descriptor, manifests, herramientas offline ni SDK; por tanto las secciones de guard/NBT de `docs/world-identity.md:36–56` no quedan verificadas por este informe. No se acredita provisión de UUID, ejecución del juego, atomicidad frente a escritores externos, N0 real, recuperación N7/N8 ni N1–N9. La aprobación se limita al marcador compartido y su integración pública en core.
