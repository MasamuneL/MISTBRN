# P2 — revisión acotada de implementación de cuarentena

Fecha: 2026-11-14, America/Mexico_City. Revisor: /root/storage_review.
Worktree: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN.
Rama de encargo: codex/alomancia-p2-native; base fed744b.
Referencia contractual: docs/execution/tasks/P2-storage-review.md.

## Dictamen

**Cumple estáticamente las correcciones solicitadas; sin hallazgos accionables bloqueantes dentro del perfil acordado.** Puede continuar la compilación y el ensayo nativo de este corte. Esta aprobación es del cambio acotado, no del backend completo ni de N0–N9. No se ejecutaron Cargo, suites, juego o pruebas Host; el build p2-native1 permanece a cargo exclusivo del principal.

No se requiere cambiar firmas públicas adicionales ni rediseñar el protocolo. La incorporación de StorageError::Quarantined y helpers privados conserva una sola autoridad económica, sin ledger secundario ni escritor de .dat.

## Identificación exacta

| Archivo revisado | SHA256 |
| --- | --- |
| crates/allomancy-horizon/src/save.rs | 00043AF15B0516BBA1CD7EB7E30568D95C031C932FF695F7304C3019866E3BD0 |
| docs/horizon-storage.md | CDD7FCC8D231B62EF610845B0E80AD50672547CB03E7D4C8E3B0FA1C91F38487 |
| integration/horizon/world_guard.rs, comprobación puntual de presencia | 4F3AD21B863141D206051C980BEAF8CD594A3606A85B22EC68031A0124EFCEEC |

save.rs se volvió a hashear al terminar la inspección y conservó el hash indicado. Los números siguientes corresponden a ese contenido, no a una build generada.

## Cumplimiento por corrección

| Contrato | Evidencia en save.rs / resultado |
| --- | --- |
| Rechazo por cualquier presencia | inspect_player:107–111 separa lectura estructural del rechazo; reject_quarantine:147–152 usa únicamente presencia. Tipo, valor, versión o layout del sibling no habilitan una excepción. No necesita state. |
| Confirmación sin ocultar Tags | inspect_player_structure:114–145 preserva mods y namespace; quarantine lo usa para ambas lecturas. Evita que Quarantined impida verificar la marca recién escrita. |
| Login ya cargado | Lectura bruta:360 precede al retorno por Entry existente:366–367; el cierre de login fija Fault ante error no Disabled. No escribe ni reinstala estado cuando detecta marca. |
| Error raw_player no vuelve a Ready | apply_credit:786–791 llama fault antes de devolver el error. El cierre:768–769 sólo cambia Busy, por lo que respeta Fault. Replay:798 queda después de la inspección. |
| Acting y mutadores normales | mutate:478 comprueba presencia antes del merge y :499 en lectura posterior; fuera de acting usa raw_player:484/502. Getter normalizado vacío no coincide con el namespace previamente instalado y se rechaza. |
| world_fault detectado | mutate:453–454 rechaza antes de Busy, lectura Host o cambio económico. Cubre las entradas lifecycle que llegan a este mutador sin volver a cargar identidad. |
| Restricción de mundo de ensayo | test_world:303–321 canonicaliza ruta Host y config con rechazo de IO; exige igualdad de rutas resultantes y WorldId. No compara strings construidos ni cambia NativeTestConfig. |
| Orden ante rollback incierto | :900–906 retiene preimagen y fija Fault; :907–915 distingue rollback no verificado, cuarentena confirmada/no confirmada. Siempre retorna error:916. |
| Marca mínima y preservación | quarantine:934–950 conserva íntegra una marca existente; si falta, añade únicamente Compound {format_version:Int(1), reason:String("rollback_unverified")}. Trabaja sobre mods ACTUALES, conservando state y claves ajenas; no restaura preimagen para marcar. |
| Setter y readback | :951 exige true para marca nueva; :954–960 vuelve a obtener snapshot del UUID esperado y compara exactamente el Compound completo. Marca existente evita setter y también exige lectura exacta. |
| Fallo de marca | Lectura inaccesible, raíz inválida/ausente, setter false o comparación distinta devuelven error; :912–913 registra quarantine_unconfirmed, sin deshacer Fault ni anunciar durabilidad. |
| Sin reentrada añadida | El helper de marca no toma Session Mutex ni llama inventario/save; usa el getter bruto y setter inline del API411 previamente auditado. No añade CAS después de la marca ni retry económico. |
| Guard por presencia | world_guard.rs:168–172 comprueba storage_fault antes de la salida por state ausente:173–174. No interpreta su contenido. Sólo se revisó esta corrección, no se reabrió la auditoría completa del guard. |

Calidad: la separación entre lectura estructural y lectura habilitante es pequeña y tiene propósito concreto; reutiliza validación de UUID, Health y Compound. La rama de error mantiene diagnóstico de fallo de marca separado del fallo original, sin convertirlo en recibo económico. No identifiqué complejidad adicional que retirar para cumplir este cambio.

## Límites que permanecen, sin ampliar el alcance

- Confirmación significa aceptación en memoria. Persistencia requiere que Horizon capture y guarde ese jugador; no equivale a fsync ni a recibo durable. Si getter/setter/readback falla, no se promete bloqueo de una reapertura. docs/horizon-storage.md lo declara correctamente.
- RollbackInventory:976–977 sigue retornando Err antes del CAS inverso: es una inyección del adaptador. Puede probar la ruta de marca usando Host real, pero no acredita un fallo real de CAS, IO o writer.
- quarantine:938 rechaza NeoForgeData ausente en lugar de reconstruirlo. Es conservador: el crédito habilitado ya exige un namespace instalado; una desaparición inesperada se informa como marca no confirmada. No se rescata desde un getter normalizado.
- Canonicalizar ambas rutas resuelve equivalencia de rutas existentes y mantiene el WorldId como segunda condición; no elimina sustituciones concurrentes, junctions alteradas después de la comprobación ni cambios externos del filesystem. Ese endurecimiento no forma parte de este encargo.
- **Rama distinta de la corregida:** CAS inicial Err + postlecturas que no verifican preimagen (:836–843) fija Fault sólo en RAM y no llama quarantine. En API411 Live, Err de la closure del CAS no instala el inventario editado; dentro del perfil sin callbacks mutantes no hay efecto económico propio que revertir en esa rama. Por eso no se presenta como bloqueante de este cambio. No extender la aprobación a anomalías de mods ajenos, desaparición del jugador o Side::File: si en un ensayo aparece incertidumbre después de un efecto real por esa ruta, no hay cobertura persistente acreditada y el caso debe quedar fallido/pendiente, no aprobarse por esta revisión.
- Se mantiene el perfil sin snapshots nuevos reentrantes durante el intervalo parcial. La lectura de código no demuestra esa exclusión ni los órdenes de lifecycle/writer en ejecución.

## Evidencia pendiente

Compilación SDK del corte y casos nativos ya exigidos por el ruling: valores arbitrarios de marca y state ausente; login existente, replay, mutador normal/acting; rollback normal sin marca; RollbackInventory con marca confirmada y reapertura rechazada; preservación de claves ajenas y marca preexistente; setter/readback realmente inaccesible como cuarentena NO confirmada; identidad fallida ya detectada; rutas canonicalizadas equivalentes/distintas. No repetir suites core para sustituir estos resultados.

Las afirmaciones WIP y de pendientes en docs/horizon-storage.md son coherentes con lo revisado. Esta revisión no acredita un resultado de build que estaba en curso.

Única escritura: este informe mediante temporal del mismo directorio y backup si existiera. Sin cambios en fuentes, SDK, manifests ni commits.


## Preparador

Revisión puntual de la decisión de límite, 2026-11-14. Contexto comunicado por el principal: SDK build2 exit0; preparación detenida antes de escribir marker porque la inspección ascendió por encima del runtime autorizado y encontró OneDrive/Documents con reparse tag 0x9000601a. Runtime y descendientes observados tienen Attributes=16. Ese resultado de build y la clasificación del tag son evidencia comunicada, no ensayos repetidos por este revisor.

**Decisión: aprobar la inspección acotada, inclusive, a AllowedRoot.** El usuario autorizó ese runtime compartido y los mundos dedicados dentro de él. Assert-NoLinks debe comprobar ReparsePoint en el objeto inicial y en cada ancestro hasta AllowedRoot; sólo después de comprobar el propio AllowedRoot puede retornar. Si nunca alcanza esa raíz, rechaza. Mantener la comprobación independiente de WorldRoot estrictamente dentro de AllowedRoot mediante prefijo terminado en separador, sin aceptar una carpeta hermana de nombre parecido. La comparación de llegada debe ser por igualdad de ruta completa normalizada, OrdinalIgnoreCase en este entorno Windows.

No hace falta reconocer o eximir tags cloud: cualquier reparse en AllowedRoot o dentro de él continúa rechazado. No auditar ancestros fuera de la raíz explícitamente autorizada cambia el alcance de la comprobación, no introduce una excepción por tipo de enlace dentro del límite. Tampoco requiere copiar o reinstalar el SDK. El contrato documental debe describir el rechazo de reparse points dentro de este límite, para no prometer una auditoría hasta la raíz del volumen.

**Hallazgo accionable de implementación mínima:** tools/prepare-world.ps1, Assert-NoLinks y su llamada desde Read-Marker. El helper recibe tanto directorios como archivos. System.IO.DirectoryInfo tiene Parent, pero System.IO.FileInfo tiene Directory y no Parent; se confirmó leyendo las propiedades del objeto real de un archivo en esta sesión (sin ejecutar la suite). Por tanto agregar el rechazo al no alcanzar AllowedRoot y conservar sólo el ascenso mediante item.Parent hará fallar la lectura de cualquier marker válido: el recorrido termina en el archivo. Ascender por Directory cuando sea FileInfo y por Parent cuando sea DirectoryInfo, o usar una operación equivalente que preserve la inspección de cada objeto. El principal recibió este hallazgo antes de cerrar la revisión.

El riesgo efectivo detectado es ese rechazo funcional de markers, no una razón para ampliar permisos o cambiar de runtime. El límite ya documentado de exclusividad operativa permanece: esta comprobación no bloquea cambios concurrentes de rutas. No se añadió un nuevo checklist ni un gate SDK. La verificación posterior del principal se registra a continuación; no fue repetida por este revisor.

Este apéndice sólo modifica el informe. No se ejecutaron Cargo, juego o suites ni se editaron fuentes por este revisor.


Estado posterior, evidencia comunicada por el principal: corregido el ascenso FileInfo.Directory / DirectoryInfo.Parent; suite del preparador tests2 exit0, caso junction aprobado, symlink no disponible por requisito de administrador. Provisión de runtime/game-p2-base/saves/AllomancyP2 exit0. El primer ensayo del cambio falló por el ascenso de FileInfo y su log se conserva. Con esa corrección, el hallazgo funcional queda resuelto según la ejecución reportada por el principal; este revisor no repitió la suite ni la provisión.