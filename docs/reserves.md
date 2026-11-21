# Reservas — contrato del núcleo (P2-core)

Fuente: `mod/src/core/src/reserves.rs`. Semántica portada sin cambios de la P2A legacy (`legacy/windows-fdc87ae:docs/reserves.md`); solo se retiró serde. Valores de ensayo, no balance definitivo.

## Tipos

- `ReserveRules { consumption_milli_per_tick:[u64;8], base_capacity_milli:[u64;8], reduction_milli:u16, capacity_bonus_milli:u16 }` en orden `Metal::ALL`. `validate()` exige consumo y capacidad positivos, reducción ≤ 200 (20 %), bonus ≤ 500 (+50 %); el exceso se rechaza, nunca se recorta. `capacity(metal) = floor(base × (1000 + bonus) / 1000)`.
- `ReserveAccounting { remainder:[u16;8] }`: fracción pendiente de una milésima, denominador fijo 1000, valores 0..1000. No es saldo económico.
- `ReserveError`: `ReductionLimitExceeded`, `CapacityBonusLimitExceeded`, `ZeroConsumption(m)`, `ZeroCapacity(m)`, `ArithmeticOverflow(m)`, `ReserveAboveCapacity(m)`, `LockedBurn(m)`, `EmptyBurn(m)`, `InvalidRemainder(m)`, `NegativeCredit`, `CapacityExceeded(m)`.

## Funciones

| Función | Comportamiento |
|---|---|
| `validate_reserves(state, acc, rules)` | Reglas, luego por metal: reserva ≤ capacidad; quemar exige desbloqueo y reserva > 0; resto < 1000; reserva 0 exige resto 0 |
| `credit_reserve(state, acc, rules, metal, i64)` | Valida todo; rechaza negativo, overflow y exceder capacidad; sin crédito parcial; 0 es idempotente |
| `consume_reserves(state, acc, rules, ticks)` | Por metal activo: `n = consumo × (1000 − reducción) × mult × ticks + resto` (u128, mult 1 encendido / 3 avivado); consume `n / 1000`, guarda `n % 1000`; al agotarse deja 0, apaga y borra el resto; devuelve los agotados en orden `Metal::ALL`. Cualquier overflow rechaza el lote entero sin cambios. Agrupar o separar ticks da el mismo resultado |
| `clear_reserves_on_death(state, acc)` | Reservas y restos a 0, todo apagado; desbloqueos y demás campos intactos |

Son helpers de lógica confiable: no autentican ni deduplican. La aplicación exactamente-una-vez de un crédito la da `persistence` (`prepare_credit`/`commit_credit`).

## Reglas por defecto del adaptador

El adaptador fija valores provisionales de 10 milésimas de consumo por tick y 100 000 milésimas de capacidad base en los ocho metales, con reducción y bonus en cero. En P3a, completar un vial acredita 25 000 milésimas al metal correspondiente si cabe el crédito completo. El desbloqueo del poder se gestiona por separado. Estos valores siguen en el código (`mod/src/bridge.rs`, `default_rules`); moverlos a datos queda pendiente.
