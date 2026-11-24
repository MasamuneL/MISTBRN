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

Orden de validación común: autoridad servidor, mundo, actor, posición finita, todos los objetivos finitos, tick no anterior a last_tick. Validación íntegra antes de cualquier mutación. Input capturado rechaza comandos excepto Extinguish (apagado de seguridad). Activate/Flare validan desbloqueo y reserva no cero. Desde P4 encienden hierro o acero (Burning o Flared) y apagan el otro de la pareja, con una señal por cambio y sin tocar reservas; repetir es idempotente. Los otros seis metales siguen rechazando NotImplemented. Drink/Purchase/Claim rechazan NotImplemented; no consumen ni premian. P1 habilita Extinguish y SelectPair: apagar solo cambia combustiones distintas de Off y emite señales por metal en orden Metal::ALL; None apaga todos. Seleccionar pareja cambia/emite únicamente si cambia; no requiere desbloquearla y no activa combustión. Repetir un comando en el mismo tick es permitido; operaciones habilitadas son idempotentes por estado.

advance_tick valida lo mismo (no input), ignora snapshot inválido y tick duplicado/anterior sin estado ni efectos; tick estrictamente nuevo actualiza únicamente last_tick. No consume reservas, aplica poderes ni recupera ticks saltados. Orden futuro de trabajo por metal queda fijado en Metal::ALL. No reloj, RNG, HashMap ni recorrido de objetivos afecta decisiones P1.

## Efectos y aplicación

`Effect { stage:EffectStage, kind:EffectKind }`; `EffectStage`: `Proposed, AcceptedInMemory`. `EffectKind`: `Physics { target:PlayerId, impulse:Vec3 }`, `Attributes { player:PlayerId, attribute:String, value_milli:i64 }`, `Inventory { transaction:TransactionId, slot:u32 }`, `Signal(Signal)`, `Persist { transaction:TransactionId }`. `Signal`: `BurnChanged { metal:Metal, burn:BurnState }`, `PairSelected { pair:Pair }`.

P1 solo emite Signal con AcceptedInMemory: la transición ya fue aceptada en el PlayerState recibido. Las demás variantes son categorías reservadas sin emisores P1; sus payloads deberán ampliarse antes de habilitar sus operaciones. Proposed nunca implica aceptación. Ni AcceptedInMemory ni Persist significan aplicado al Host o durable: el núcleo no contiene recibo de Host ni fsync. El adaptador debe mantener copia anterior y efectos pendientes durante aplicación; si falla, restaurar estado y deshacer aplicaciones parciales antes de confirmar. Si no puede deshacer, bloquear y mantener explícitamente pendiente/error; no confirmar éxito ni repetir premios. En P1 las señales no ocasionan inventario, daño, atributos ni XP.

## Metales y física (P4)

`is_metal(id) -> bool` y `is_coin(id) -> bool` (`metals.rs`) clasifican ids `namespace:path` de bloques e ítems: lista explícita y reglas de nombre sobre `minecraft:` (`iron`, `gold`, `copper`, `netherite_`, `chainmail_`, `_bucket`, `_minecart`), sin comida, huevos de aparición ni plantillas; del mod, lingotes, virutas, `raw_tin`, `raw_zinc`, `clip`, `boxing` y `bronze_earring`. Otros espacios de nombres no son metal.

`physics` (módulo público, solo geometría en f64): `step(&ForceRules, &StepInput) -> Result<StepOutcome, StepError>` calcula un tick de fuerza; `StepError`: `NonFinite`, `OutOfRange`, `Coincident`. `ForceRules::{BURNING, FLARED}` llevan aceleración, alcance, topes, masa de ítem, umbral de ancla y amortiguación de elevación. `client_velocity` corrige la velocidad reportada por el cliente con la gravedad y el arrastre vanilla; `coin_damage(speed)` da el daño de una moneda. El núcleo no aplica nada: el adaptador lee posiciones del servidor y entrega las velocidades al motor, que resuelve la colisión.

## Eventos y funciones reservadas

`EventKind`: `ResearchObserved { research:ResearchId }`, `MilestoneReached { key:String }`, y desde P5 `ResearchEvidence { research:u16, evidence:u8 }` y `PracticeAction { category:u8 }` (códigos del catálogo de `progress.rs`, nunca texto del cliente). `VerifiedEvent` tiene campos privados `id:EventId, tick:Tick, world_id:WorldId, actor:PlayerId, kind:EventKind`; getters `id, tick, world_id, actor, kind`. Sin codec. Constructor `from_server_observation(id:EventId, kind:EventKind, ctx:&WorldSnapshot) -> Result<VerifiedEvent,RuleError>` valida autoridad y geometría y captura identidad/tick; debe llamarlo exclusivamente lógica confiable tras observar el Host. Este chequeo estructural no prueba por sí mismo un hito ni convierte texto del cliente en evidencia. P1 no observa ni reclama progresión.

P5 implementa el progreso sobre `SavedPlayer`, no sobre `PlayerState` ni `apply_command`: `observe(&mut SavedPlayer,&VerifiedEvent,game_time)->Result<Observed,SaveError>` y `prepare_purchase`/`prepare_conclude`/`prepare_hint` (ver `docs/persistence.md`); nodos e investigaciones se identifican con códigos `u16` estables del catálogo (`CATALOG`), no con `NodeId`/`ResearchId`, y `Command::Purchase`/`Claim` siguen sin uso. Reservada para P6, **no implementada**: `claim_milestone(&mut PlayerState,&VerifiedEvent)->Result<Reward,RuleError>`. El guardado se implementó en P2-core por jugador, no por `WorldState`: `encode_save(&SavedPlayer)` y `decode_save(&[u8], &WorldId, &PlayerId)` (ver `docs/reserves.md` y `docs/persistence.md`). Los tipos de catálogos/resultados se definirán en sus fases; no hay stubs que aparenten soporte.

## Gate P2 de persistencia e inventario

Backend decidido (ver `docs/plan.md`, P2-native): bytes del codec propio en `NeoForgeData.PlayerPersisted.mistbrn` como `ByteArray`, leyendo y reescribiendo el compound completo y preservando claves ajenas. Inventario y `persistent_data` se modifican juntos dentro de `host.transaction`; cualquier `Err` aborta ambos. El getter del SDK normaliza una raíz `NeoForgeData` no-compound a vacía; el adaptador comprueba además `entity().nbt()` antes de escribir, y si esa comprobación detecta una raíz corrupta lo prueba S11 (P2-native). `Effect::Persist` no es por sí mismo durabilidad: la da el writer del juego, probado en P2-native.
