# SDD ledger — plan: docs/superpowers/plans/2026-11-12-alomancia.md
Fecha: 2026-11-12 America/Mexico_City. Base sin HEAD, checkout normal master.
Ruling: investigar P0 en workspace explícitamente solicitado; no existe commit para worktree. Crear aislamiento de código después de C0. No implementar sobre master.
Ruling: ledger recuperable en docs/execution en vez de scratch bash; Windows sin bash confirmado, checkpoints del usuario son autoridad. No borrar artefactos de recuperación.
Ruling: tareas con propietarios disjuntos pueden ejecutarse en paralelo conforme autorización explícita del usuario y plan.
Ruling: aprobación de diseño/plan recibida; no reabrir brainstorming.

## Preflight por tarea
| Tarea | Coherencia / resolución |
|---|---|
| P0 | Hooks y ejecución real obligatorios; fuentes estáticas no sustituyen prueba. |
| P1 | Depende C0; contratos antes de consumidores. |
| P2 | Economía y persistencia; no confirmar inventario sin transacción del adaptador. |
| P3 | Datos tras P1; integración vial requiere P2. |
| P4 | Requiere P2/P3; física por colisión del motor. |
| P5 | Depende P2; diario aquí es lógica; representación en P7. |
| P6 | Loot se transfiere de P3; no edición paralela. |
| P7 | P4/P5; C2 solo tras interfaz y ciclo real verificados. |
| P8 | Espera C2; tests separados para evitar colisiones. |
| P9 | Cooperativo condicionado a P0; individual sigue alcance aprobado. |
| P10 | C3 y decisión P9; no empaquetar sin integración real. |

## Interfaces y archivos compartidos
| Productor/consumidor | Interfaz/archivo | Decisión |
|---|---|---|
| P0 / P1–P10 | SDK revisión / compatibilidad | C0 bloquea dependientes. |
| P1 / P2–P9 | tipos, comandos, state, contratos | Congelar antes de delegar; dueño principal. |
| P2 / P3–P9 | reservas / guardado | Integrar serialmente con inventario. |
| P3 / P4,P6 | loot.json y materiales | Transferencia explícita después P3. |
| P4 / P7 | input.rs y efectos | Principal único escritor input.rs. |
| P5 / P6–P9 | skills/research catálogos | Congelar antes consumidores. |
| P6 / P9 | autoridad hitos | Principal integra red. |
| P7 / P8,P10 | UI, es_es.json | Transferir propiedad de assets después P7. |
| P8 / P4,P9 | powers/mod.rs y tests/powers.rs | Integración serial principal; archivos por pareja. |
| P1–P10 entre sí | Cargo.toml, Cargo.lock | Propiedad exclusiva principal. |

## Ejecución
- P0 in_progress. Inspección inicial: solo tres docs; sin HEAD. Clone exit 128 DNS. Conector GitHub funciona; revisión 36eb98b6f6ec4f22649cbe65bbfd32bc0dc2083f fijada.
- Inventario delegado a Terra medium /root/p0_inventory; propietario exclusivo de ficha e informe P0-inventory.
- Próximo: análisis estático de hooks y cotejo instalación. Sin pruebas reales aprobadas.

P0 parcial: build mínimo exit0 y factory cargada. Smoke incompleto. Candidate integrado sin compilar; revisión automática y revisor bloqueados por cuota. GitHub público autorizado MasamuneL/MISTBRN; publicación curada en curso. No declarar C0.
P0-capabilities: fix round1/5 (sound category corregida por Terra;re-review y smokeGREEN pendientes). V3baseline ping/ack válidos y soundfalse demostrados; Contentfixture timeout aislado antes tablas;Luna investiga sin cambios.
P0: fix round1/5 (R1/R2 ADDRESSED;re-reviewAstra limpio). C0aprobado,commit/publicaciónpendientes. P2debevalidarraízNBToriginal fuera de turno,rollback,replay,cortes;ninguna promesa durableporACK.