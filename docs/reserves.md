# Reservas P2A — contrato congelado

2026-11-13. API propia del núcleo, reexportada desde `allomancy_core`. Parámetros configurables de ensayo; no balance definitivo. No cambia `PlayerState`, comandos P1 ni `advance_tick`.

## Datos y unidades

`ReserveRules { consumption_milli_per_tick:[u64;8], base_capacity_milli:[u64;8], reduction_milli:u16, capacity_bonus_milli:u16 }` tiene `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`, sin valores por defecto. Cada índice sigue `Metal::ALL`. Consumo/capacidad base deben ser positivos. Reducción global entre 0 y 200 (20%); bonus de capacidad entre 0 y 500 (+50%). Un exceso se rechaza, no se recorta. `validate(&self) -> Result<(),ReserveError>` comprueba las ocho reglas y desbordamiento de capacidad. `capacity(&self, metal:Metal) -> Result<u64,ReserveError>` valida las reglas y devuelve `floor(base_capacity_milli * (1000 + capacity_bonus_milli) / 1000)`.

`ReserveAccounting { remainder:[u16;8] }` tiene `Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize`. Cada resto es una fracción de milésima con denominador fijo 1000, en 0..1000 exclusivo. `Default` pone los ocho restos a cero. No es un saldo económico adicional ni cambia la unidad de reserva. El denominador no depende de reglas ni del framerate, por lo que cambiar una reducción conserva correctamente el consumo pendiente. Una tasa de ensayo posible es 1 milésima/tick y capacidad 1000 para cada metal; el llamador debe escogerlas explícitamente.

`ReserveError` tiene `Clone, Copy, Debug, PartialEq, Eq`, variantes `ReductionLimitExceeded`, `CapacityBonusLimitExceeded`, `ZeroConsumption(Metal)`, `ZeroCapacity(Metal)`, `ArithmeticOverflow(Metal)`, `ReserveAboveCapacity(Metal)`, `LockedBurn(Metal)`, `EmptyBurn(Metal)`, `InvalidRemainder(Metal)`, `NegativeCredit`, `CapacityExceeded(Metal)`.

## Funciones

```rust
validate_reserves(&PlayerState, &ReserveAccounting, &ReserveRules)
    -> Result<(), ReserveError>
credit_reserve(&mut PlayerState, &ReserveAccounting, &ReserveRules,
               Metal, amount_milli:i64) -> Result<(), ReserveError>
consume_reserves(&mut PlayerState, &mut ReserveAccounting, &ReserveRules,
                 elapsed_ticks:u64) -> Result<Vec<Metal>, ReserveError>
clear_reserves_on_death(&mut PlayerState, &mut ReserveAccounting)
```

`validate_reserves` valida reglas primero y luego cada metal en orden `Metal::ALL`: reserva no superior a capacidad; quemar requiere desbloqueo y reserva positiva; resto menor que 1000; reserva vacía requiere resto cero. Permite reservas en metales bloqueados apagados y restos pendientes en metales apagados con reserva positiva. Serde por sí solo no valida.

`credit_reserve` valida todo el estado/reglas/restos, rechaza crédito negativo, suma con overflow comprobado y rechaza capacidad excedida. Cero es idempotente. No cambia combustión ni desbloquea metales ni borra restos; no recorta ni acepta parcialmente un crédito.

`consume_reserves` valida primero y calcula todos los resultados antes de aplicar cualquiera. Por metal activo: `numerador = consumo_base_por_tick * (1000 - reducción) * multiplicador * elapsed_ticks + resto`; multiplicador 1 normal y 3 avivado. Consume `floor(numerador/1000)` milésimas y conserva `numerador%1000`. Usa aritmética comprobada `u128` para numeradores y exige cociente representable en `u64`; cualquier overflow rechaza el lote completo intacto, aun cuando una reserva finita se agotaría. Un metal apagado conserva reserva/resto. Cero ticks no cambia nada. Al alcanzar o superar exactamente la reserva, la deja a cero, apaga y elimina su resto; devuelve únicamente esos metales agotados en orden `Metal::ALL`. Agrupar ticks o separarlos conserva resultado y resto mientras no haya otras transiciones intermedias. No modifica `last_tick`; el integrador calcula ticks transcurridos sin cobrar tiempo offline y evita replay/duplicación de ticks.

`clear_reserves_on_death` vacía ocho reservas/restos y apaga, preservando los demás campos, incluidos desbloqueos. Es una limpieza idempotente que admite reparar reservas inconsistentes. El codec debe persistir estado y contabilidad juntos bajo identidad estable de jugador/mundo. Al reconectar debe apagar combustiones y conservar restos; no reconstruir la contabilidad con `Default` para estados existentes. La función de muerte es el único reinicio de restos de ciclo de vida definido aquí; el agotamiento también elimina el resto del metal vacío.

Todas las operaciones son helpers internos para lógica confiable. No autentican clientes, no verifican inventario ni `TransactionId`, no emiten recibos de Host ni prometen durabilidad. El adaptador debe validar contexto servidor, identidad, ticks, transacciones e inventario antes de invocarlas. Créditos repetidos se aplican repetidamente: deduplicación y rollback de Host pertenecen a P2B/integración. P2A no habilita beber ni poderes y avivado x2 potencia queda en P4.

Integración P2B: `ReserveRules` y `ReserveAccounting` usan `serde(deny_unknown_fields)` para rechazar claves desconocidas al deserializar guardados. No cambia sus campos, reglas ni funciones; serde sigue sin sustituir `validate_reserves`.
