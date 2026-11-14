# P3 — revisión independiente parcial de catálogo y validador

Fecha: 2026-11-13 America/Mexico_City. Base observada: 5f679d06297cddef25031d239df95b4693fca5d0; rama codex/alomancia-p2. Revisor delegado independiente de datos/validador; modelo exacto no expuesto. Alcance: content/{metals,recipes,loot}.json, docs/content-catalog.md, tools/{validate-content,test-content-validation}.ps1 y evidencia asociada. No se revisa ni acredita P3 integrado, jugable, loot/comercio real, SDK, consumo de inventario ni bridge P2/P3.

## Dictamen

**Cambios solicitados en el validador: dos hallazgos P2.** El catálogo actual es coherente como borrador interno y no contiene los valores erróneos de las reproducciones. No procede dar aprobación conjunta definitiva datos+validador hasta resolver los dos huecos del gate. No hay hallazgos P0/P1 ni petición de integrar con SDK para validar estos JSON propios.

## Hallazgos accionables

### [P2] Exigir tipos booleanos antes de comparar valores

`tools/validate-content.ps1:23`, `:26` y `:27` comparan campos JSON con `$true`/`$false` usando `-cne`, sin comprobar `-is [bool]`. PowerShell convierte el operando derecho según el tipo izquierdo: las cadenas JSON `"True"` y `"False"` se aceptan como si fueran booleanos. Reproducción: en copia de metals.json fijar primer hito `claim_once_per_player="True"` y `requires_warden_kill="False"`; en loot.json fijar primera fuente `verified="False"`. El validador termina **exit 0**. El esquema documentado exige booleanos (docs/content-catalog.md:27–30); un consumidor PowerShell que evalúe la cadena `"False"` por truthiness obtiene true. Pedir tipo booleano explícito en hitos, sources y trades, y caso negativo sobre JSON real. No hace falta biblioteca de esquema ni nueva abstracción.

### [P2] Distinguir grafo acíclico de mera alcanzabilidad

`tools/validate-content.ps1:30–31` calcula un cierre de disponibilidad; detecta ciclos sin fuente, pero no un ciclo alcanzable. Reproducción: añadir una receta con ID único `allomancy:recycle_iron`, ingrediente `allomancy:steel_ingot` count 1 y resultado `allomancy:iron_ingot` count 1. Las recetas existentes iron→steel más la nueva steel→iron forman un ciclo; todas son alcanzables desde la receta vanilla→iron y el validador termina **exit 0**. El plan P3 exige obtención sin dependencias circulares (docs/superpowers/plans/2026-11-12-alomancia.md:142). El negativo llamado cycle solo quita la fuente inicial (`tools/test-content-validation.ps1:20`) y comprueba inalcanzabilidad; no cubre un ciclo alcanzable. Añadir una comprobación sencilla de ciclos del grafo ingrediente→resultado y un negativo independiente que conserve las fuentes. Mantener además el cierre de alcanzabilidad actual, que resuelve un requisito distinto.

## Observaciones acotadas

- El validador tiene 36 líneas con sentencias muy largas, en particular :23–30; resulta difícil cotejar cada regla y localizar fallos. Al corregir los dos hallazgos conviene desplegar esos bloques en líneas normales, manteniendo funciones PowerShell locales y cero dependencias. La longitud/minificación sola no es un bloqueo ni justifica un framework.
- `grants: allomancy:final_mastery` es un identificador simbólico terminal sin otro registro en los tres JSON. El validador comprueba únicamente su sintaxis, mientras docs/content-catalog.md:38 habla de resolver toda referencia de hito. Conviene aclarar en el contrato que grants pertenece a un dominio simbólico provisional separado, o declarar sus valores admitidos, antes de que P6 lo consuma. No se presume que sea un item ni una API de Horizon.
- Como son parámetros de ensayo y no validación de economía/runtime, aceptar una receta con resultado de dos viales o un requires_warden_kill contradictorio solo en una fuente no prueba duplicación en juego ni soporte de hitos. Estos probes no se elevan a hallazgos adicionales de alcance. El catálogo actual fabrica un vial por un envase+muestra y todos los indicadores Warden son false; las recetas no consumen viales llenos.

## Verificación independiente ejecutada

| Comprobación | Resultado |
| --- | --- |
| pwsh -NoProfile -File tools/test-content-validation.ps1 | Exit 0; baseline y nueve negativos PASS. Fixtures preservadas target/content-validation-tests/1ef83942619e479ab0b328b96ceef888. |
| Parser AST PowerShell de los dos scripts | Cero errores en ambos. |
| Kahn sobre grafo actual ingrediente→resultado | DAG=True, 33/33 nodos visitados. |
| Tipos booleanos actuales de hitos/sources/trades | Booleanos reales en todos los campos comprobados. |
| Fixtures boolean-strings y reachable-cycle | Ambas aceptadas exit 0, demostrando los hallazgos. |
| Fixtures exploratorias two-vials y warden-source | Exit 0; límites de alcance descritos arriba, sin nuevo claim runtime. |

Las cuatro fixtures exploratorias y result.log quedan preservadas bajo target/content-validation-tests/p3-review-edeecf4bde894784bac9a5605a263418. La suite existente muta/copía JSON real y comprueba exit code más mensaje, sin búsqueda de texto fuente como sustituto de comportamiento. Los logs RED/GREEN/AST previos se inspeccionaron; RED histórico acredita nueve negativos fallidos ante stub que aceptaba todo. No se repitió ese stub sobre implementación ajena.

Datos actuales cotejados: ocho metales en orden hierro/acero, estaño/peltre, zinc/latón, cobre/bronce; cuatro parejas, 32 items, cuatro hitos con reclamo único y sin Warden, 26 recetas. Cantidades enteras positivas, referencias materiales declaradas y cierre de recetas completo desde siete fuentes declaradas; dos trades propuestos. Loot/trade permanecen proposed/unverified y la salida advierte explícitamente que alcanzar desde propuestas no demuestra SDK ni disponibilidad real. Libro, mesa, monedas, muestras y viales están presentes. No se fijan poderes ni balance final.

## Hashes del corte revisado (SHA256)

| Archivo | SHA256 |
| --- | --- |
| content/metals.json | 1FD94EAC4E99AF9A4BD4F3D465DEFB43F59CC45B1920A0E6DE8CB4A18BA39EC9 |
| content/recipes.json | CF14DAF497B39E5F63AC3A1553DE199E16BF6FE9B700C3B33FBE1EA71D3B6AAF |
| content/loot.json | BA9BFFAC20FF3ED2358B192E15E2BAA72CC9A563DE1DF542D0E7A172860B21BC |
| docs/content-catalog.md | 41B85D02C85DF53A2E5C6D310DDE5C85CFADF87A458AB6DF1659DFD6648C7D69 |
| tools/validate-content.ps1 | 7FDC2093325E06371BE98CA55F7DD2AC2F8F763EFCA178B7A4F2181C13B07550 |
| tools/test-content-validation.ps1 | 7AC91BAB01E177ACBA083D8B09D3D406998A65F1649B424A5EBC3718F706ABAA |

## Siguiente acción

Principal devuelve solo los dos hallazgos P2 al propietario del validador. Corregir tipos booleanos y detección de ciclos con negativos reales; ampliar bloques para legibilidad sin introducir dependencias. Resolver la observación grants mediante contrato mínimo antes de consumirla. Revisión posterior cotejará el diff y los casos nuevos; no requiere Cargo, native build ni Horizon. P3 completa conserva pendientes inventario/vial atómico, assets, loot retroactivo, fabricación/beber en juego y bridge con P2.
## Re-review fix1 — dictamen vigente (HEAD67f4d76)

Revisión estrictamente acotada al diff de tools/validate-content.ps1, tools/test-content-validation.ps1 y docs/content-catalog.md contra target/review/p3-fix1-before. No reinicio de auditoría general; datos JSON inmutados. Hashes del fix1 cotejados exactamente con la entrega:

- validator: EB3B2A63B1AFB08300BDFD852BCC3E0F51BB3B7332AD5EF043DE723466DB1554
- tests: 35E1FE2A88562543DD892FC63F0CB888D4F9BE5FA26B07BD2AD0046FB2A78A1E
- doc: 537C606550727BC582CF3D0EB4641CEE5EF36720D51540553950AE8B14F7D302

| Hallazgo/observación anterior | Estado fix1 | Evidencia |
| --- | --- | --- |
| P2: booleanos coercionados | ADDRESSED | Guards -isnot [bool] explícitos en claim/warden/source/trade; cuatro negativos JSON separados fallan con exit1. |
| P2: ciclo alcanzable | ADDRESSED | Kahn con aristas ingrediente→resultado deduplicadas, separado del cierre; reachable_cycle falla con mensaje de ciclo; cycle anterior sigue rechazado por inalcanzabilidad. |
| Legibilidad sin dependencias | ADDRESSED proporcionalmente | Las sentencias antes separadas por punto y coma se despliegan; algoritmo de grafo local, sin dependencia/framework nuevo. La indentación puede mejorar, pero no constituye bloqueo nuevo. |
| grants: dominio simbólico provisional | Documentación ADDRESSED; validación NOT ADDRESSED completamente | Enum único documentado, pero la nueva comprobación introdujo coerción de tipo descrita abajo. |

### Nueva rotura del arreglo: [P2] Conservar comprobación string para grants

La versión anterior pasaba grants a Id, que rechazaba valores no string. Fix1 lo sustituyó por `$null -ne $h.grants -and $h.grants -cne 'allomancy:final_mastery'`. Si grants es el booleano JSON true, PowerShell convierte el literal no vacío del lado derecho a true; `true -cne true` es false y no rechaza el campo. Reproducción exclusiva del cambio: copiar catálogo y fijar el último hito grants=true. **GRANT_BOOL_TYPE=Boolean; GRANT_BOOL_EXIT=0.** El campo incumple el enum simbólico documentado, y es una regresión respecto del guard de sintaxis/tipo anterior. Corrección mínima: comprobar -isnot [string] además del literal permitido y añadir negativo grants=true; no requiere extender registro ni tocar SDK.

Fixture preservada: target/content-validation-tests/p3-fix1-review-551e069599c645bf8d4fd5a774e7520f (metals.json y result.log). La aceptación se volvió a comprobar directamente con el validador fix1. Ninguna implementación fue modificada por el revisor.

### Gates independientes fix1

Suite actual: `pwsh -NoProfile -File tools/test-content-validation.ps1`, **16/16 PASS, exit0**; log completo target/p3-fix1-review-green.log, fixtures target/content-validation-tests/e4704954420a47c6a3ecd69835619947. Ambos parsers AST: cero errores. Logs TDD del propietario cotejados: los seis casos nuevos aceptados erróneamente ante versión anterior en target/p3-fix1-red.log y luego los 16 pasan en target/p3-fix1-green.log. Un primer lanzamiento de revisión produjo salida truncada de herramienta aunque sus fixtures terminaron; se repitió solo para obtener salida completa y exit code verificables, sin mutar fuentes.

**Dictamen vigente fix1: changes_requested por una regresión P2 del propio arreglo; ambos P2 originales cerrados.** Siguiente acción exacta: propietario conserva tipo string en enum grants y agrega su negativo; re-review posterior solo de ese diff/caso. No ampliar auditoría general. La aprobación futura corresponderá únicamente a catálogo+validador internos; P3 jugable/runtime/loot/bridge permanece pendiente.
## Re-review fix2 — dictamen final vigente: APPROVED (alcance interno parcial)

HEAD67f4d76; re-review estrictamente limitado al guard grants y tests/documentación correspondientes contra target/review/p3-fix2-before. No nuevo audit general ni revisión de core/P2/SDK. Se cotejaron exactamente los hashes de entrega:

- tools/validate-content.ps1: 0BCA5616C893769959E38A152CF8BB00020F92AFF2094705081E470980330BFF
- tools/test-content-validation.ps1: 1147845A92D3247677471C2D30BFA02B5F9058D0A92C47B6949B908FB029694C
- docs/content-catalog.md: 673849A1E3CD5D27ECEFEEF02C60FA5ED062D704804D33F84585736E9E06CB18

**Regresión grants de fix1: ADDRESSED.** La condición comprueba existencia de la propiedad y, cuando existe, tipo string antes del literal permitido. La ausencia queda permitida; null explícito, booleanos, números y cadena vacía se rechazan, conforme al contrato actualizado. La comparación ya no acepta true por coerción. No se identificó nueva rotura introducida por este diff. Ambos P2 originales y las observaciones aceptadas permanecen ADDRESSED según fix1.

Verificación independiente fresca: `pwsh -NoProfile -File tools/test-content-validation.ps1` **21/21 PASS, exit0**; AST de ambos scripts **0 errores**. Baseline valida el literal correcto y ausencia de grants en los otros hitos; los cinco nuevos negativos true/false/null/0/vacío terminan exit1 con mensaje esperado. Log target/p3-fix2-review-green.log; fixtures preservadas target/content-validation-tests/d6cac8f7af8d47158b8286bd196154f6. Logs del propietario target/p3-fix2-red.log/green.log cotejados: antes true/null se aceptaban erróneamente; después todos los 21 casos pasan. Datos de contenido permanecieron inmutados durante re-review.

**Spec compliance y calidad/correctitud: approved para catálogo+validador internos P3.** Este dictamen sustituye el changes_requested de las rondas anteriores únicamente tras los fixes revisados. No hay hallazgos accionables pendientes dentro de ese alcance parcial. El catálogo conserva su carácter provisional, fuentes propuestas sin verificar y advertencia explícita de que la alcanzabilidad interna no acredita SDK/runtime.

Siguiente acción exacta del principal: comprobar corte/hashes y preparar commit parcial separado de datos+validador con sus informes. P3 completa NO aprobada: siguen pendientes bridge Horizon/P2, consumo atómico real del vial/inventario lleno, fabricación/beber en juego, assets y obtención retroactiva real de loot/comercio/hitos. Sin Cargo, launcher, SDK, cambios de implementación, commits ni subagentes por el revisor. Solo informes propios y fixtures/log ignorados; .bak preservado.