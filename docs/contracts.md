# Contratos del núcleo — Masamune's MISTBRN

Contrato P1 congelado 2026-11-13; revisado en P0b (2026-11-21). Son tipos del proyecto, no APIs del SDK. Fuente normativa: `mod/src/core/src/{types,state,commands}.rs`, reexportados en la raíz `allomancy_core`.

**Cambio P0b:** el núcleo no tiene dependencias de registro. Se retiraron `Serialize/Deserialize` de todos los tipos: ningún tipo del núcleo se decodifica de bytes externos salvo mediante los codecs propios y validados que se definan (guardado en P2, peticiones de cliente en P7).

## Identidad y cantidades

`PlayerId`, `WorldId`, `NodeId`, `ResearchId`, `EventId`, `TransactionId`: newtypes opacos de `String` privada, `new(impl Into<String>) -> Self`, `as_str() -> &str`, orden lexicográfico. No interpretan su contenido ni prueban autenticidad. El adaptador usa `WorldId` constante `local`: el aislamiento entre mundos lo dan los ficheros de jugador por mundo del juego. `Tick(pub u64)`.

`Metal` en orden fijo: `Iron, Steel, Tin, Pewter, Zinc, Brass, Copper, Bronze`; `Metal::ALL: [Metal;8]`, `index() -> usize`, `pair() -> Pair`. `Pair`: `IronSteel, TinPewter, ZincBrass, CopperBronze`. `Vec3 { x:f64, y:f64, z:f64 }`, `is_finite() -> bool`; solo geometría admite flotantes.

`BurnState`: `Off, Burning, Flared`. `MetalState { unlocked:bool, reserve_milli:u64, burn:BurnState }`; milésimas enteras, sin consumo P1. `PlayerState { player_id:PlayerId, world_id:WorldId, last_tick:Option<Tick>, selected_pair:Pair, metals:[MetalState;8] }`. `PlayerState::new(player_id, world_id)` inicia metales bloqueados, reservas cero, apagados, pareja IronSteel y last_tick None. `WorldState { world_id:WorldId, players:BTreeMap<PlayerId,PlayerState> }`. Sin decodificador genérico: solo el codec de guardado de P2 produce estos tipos desde bytes, tras validarlos.

## Frontera de confianza

`Authority`: `Client, Server`; `InputContext { gameplay_allowed:bool }`; `TargetSnapshot { id:String, position:Vec3 }`; `WorldSnapshot { tick:Tick, world_id:WorldId, actor:PlayerId, position:Vec3, targets:Vec<TargetSnapshot>, authority:Authority, input:InputContext }`. Ninguno tiene decodificador. Solo el adaptador confiable crea snapshots del Host; nunca copiar `authority` de una petición cliente. `gameplay_allowed` proviene del foco/chat/menú comprobado por el adaptador.

`Command`: `Drink { transaction:TransactionId, slot:u32 }`, `Activate { metal:Metal }`, `Flare { metal:Metal }`, `Extinguish { metal:Option<Metal> }`, `Purchase { node:NodeId, transaction:TransactionId }`, `Claim { event:EventId, transaction:TransactionId }`, `SelectPair { pair:Pair }`. Un EventId del cliente es una referencia, nunca evidencia verificada.

`Operation`: `Drink, Activate, Flare, Purchase, Claim`. `RuleError`: `ServerAuthorityRequired, WorldMismatch, PlayerMismatch, NonFinitePosition, NonFiniteTarget, StaleTick, InputCaptured, MetalLocked(Metal), EmptyReserve(Metal), NotImplemented(Operation)`.

```rust
apply_command(&mut PlayerState, Command, &WorldSnapshot) -> Result<Vec<Effect>, RuleError>
advance_tick(&mut PlayerState, &WorldSnapshot) -> Vec<Effect>
```

Orden de validación común: autoridad servidor, mundo, actor, posición finita, todos los objetivos finitos, tick no anterior a last_tick. Validación íntegra antes de cualquier mutación. Input capturado rechaza comandos excepto Extinguish (apagado de seguridad). Activate/Flare validan desbloqueo y reserva no cero, después rechazan NotImplemented. Drink/Purchase/Claim rechazan NotImplemented; no consumen ni premian. P1 habilita Extinguish y SelectPair: apagar solo cambia combustiones distintas de Off y emite señales por metal en orden Metal::ALL; None apaga todos. Seleccionar pareja cambia/emite únicamente si cambia; no requiere desbloquearla y no activa combustión. Repetir un comando en el mismo tick es permitido; operaciones habilitadas son idempotentes por estado.

advance_tick valida lo mismo (no input), ignora snapshot inválido y tick duplicado/anterior sin estado ni efectos; tick estrictamente nuevo actualiza únicamente last_tick. No consume reservas, aplica poderes ni recupera ticks saltados. Orden futuro de trabajo por metal queda fijado en Metal::ALL. No reloj, RNG, HashMap ni recorrido de objetivos afecta decisiones P1.

## Efectos y aplicación

`Effect { stage:EffectStage, kind:EffectKind }`; `EffectStage`: `Proposed, AcceptedInMemory`. `EffectKind`: `Physics { target:PlayerId, impulse:Vec3 }`, `Attributes { player:PlayerId, attribute:String, value_milli:i64 }`, `Inventory { transaction:TransactionId, slot:u32 }`, `Signal(Signal)`, `Persist { transaction:TransactionId }`. `Signal`: `BurnChanged { metal:Metal, burn:BurnState }`, `PairSelected { pair:Pair }`.

P1 solo emite Signal con AcceptedInMemory: la transición ya fue aceptada en el PlayerState recibido. Las demás variantes son categorías reservadas sin emisores P1; sus payloads deberán ampliarse antes de habilitar sus operaciones. Proposed nunca implica aceptación. Ni AcceptedInMemory ni Persist significan aplicado al Host o durable: el núcleo no contiene recibo de Host ni fsync. El adaptador debe mantener copia anterior y efectos pendientes durante aplicación; si falla, restaurar estado y deshacer aplicaciones parciales antes de confirmar. Si no puede deshacer, bloquear y mantener explícitamente pendiente/error; no confirmar éxito ni repetir premios. En P1 las señales no ocasionan inventario, daño, atributos ni XP.

## Eventos y funciones reservadas

`EventKind`: `ResearchObserved { research:ResearchId }`, `MilestoneReached { key:String }`. `VerifiedEvent` tiene campos privados `id:EventId, tick:Tick, world_id:WorldId, actor:PlayerId, kind:EventKind`; getters `id, tick, world_id, actor, kind`. Sin codec. Constructor `from_server_observation(id:EventId, kind:EventKind, ctx:&WorldSnapshot) -> Result<VerifiedEvent,RuleError>` valida autoridad y geometría y captura identidad/tick; debe llamarlo exclusivamente lógica confiable tras observar el Host. Este chequeo estructural no prueba por sí mismo un hito ni convierte texto del cliente en evidencia. P1 no observa ni reclama progresión.

Firmas del plan reservadas, **no exportadas ni implementadas**: `observe(&mut PlayerState,&VerifiedEvent,&ResearchCatalog)->Vec<Discovery>`, `purchase_node(&mut PlayerState,NodeId,&SkillCatalog)->Result<(),RuleError>`, `claim_milestone(&mut PlayerState,&VerifiedEvent)->Result<Reward,RuleError>`, `encode_save(&WorldState)->Result<Vec<u8>,SaveError>`, `decode_save(&[u8])->Result<WorldState,SaveError>`. Los tipos de catálogos/resultados se definirán en sus fases; no hay stubs que aparenten soporte.

## Gate P2 de persistencia e inventario

Backend decidido (ver `docs/plan.md`, P2-native): bytes del codec propio en `NeoForgeData.PlayerPersisted.mistbrn` como `ByteArray`, leyendo y reescribiendo el compound completo y preservando claves ajenas. Inventario y `persistent_data` se modifican juntos dentro de `host.transaction`; cualquier `Err` aborta ambos. El getter del SDK normaliza una raíz `NeoForgeData` no-compound a vacía; la guarda que lo compensa depende del gate G7 de P0b. `Effect::Persist` no es por sí mismo durabilidad: la da el writer del juego, probado en P2-native.
