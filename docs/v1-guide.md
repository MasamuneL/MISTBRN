# Guía V1: recorrido a mano

Demo interna de hierro y acero. Tu evaluación es el criterio de cierre de V1: apunta lo que notes en la tabla del final.

## Preparación

- Instancia `mistbrn-dev` en el launcher de Horizon con el build de la PR (lo instala `tools/build-native.sh`).
- Crea un mundo nuevo en **Supervivencia**, con trucos activados para darte los materiales.
- No definas `MISTBRN_TEST_SCENARIO`: con ella el juego no es el de un jugador.

Materiales (por chat): `/give @s glass 2`, `/give @s iron_ingot 3`, `/give @s cobblestone 1`, `/give @s coal 2`, `/give @s book 1`, `/give @s iron_block 1`.

## Recorrido

1. **Fabricar el vial** en la cuadrícula 2×2 del inventario:
   - 2 cristales, uno encima del otro → 2 viales vacíos.
   - Lingote de hierro + adoquín → 2 virutas de hierro.
   - Vial vacío + virutas de hierro → vial de hierro.
2. **Fabricar la cartilla** (también en la cuadrícula 2×2):
   - Lingote de hierro + 2 carbones → lingote de acero.
   - Libro + lingote de hierro + lingote de acero → «Cartilla: hierro y acero».
   - La cartilla es provisional: la progresión de P5/P6 la sustituirá.
3. **Beber el vial**: mantén clic derecho hasta que termine. Recuperas el vial vacío y la reserva de hierro sube a 25 000 (aún sin panel: el hierro está bloqueado).
4. **Estudiar la cartilla**: mantén clic derecho. El chat dice «Alomancia: ya puedes quemar hierro y acero.» y aparece el panel arriba a la izquierda, con la barra de hierro al 25 %. Una segunda cartilla no se consume: «…ya conoces el hierro y el acero; conservas la cartilla.»
5. **Tirón**: coloca el bloque de hierro a unos 10 bloques, míralo y pulsa **Z** para quemar hierro. Te acerca al bloque y te detiene en su cara. Con **R** avivas (más fuerza, triple gasto); otra R vuelve a quemar normal.
6. **Apagar**: pulsa **G** (apaga todo) o otra vez **Z** (solo hierro). La fuerza para.
7. **Acero** (opcional): con **X** quemas acero, que empuja en sentido contrario. No tienes reserva de acero: date `/give @s iron_ingot 1`, `/give @s coal 2` y `/give @s cobblestone 1`, fabrica otro lingote de acero, virutas de acero (lingote de acero + adoquín) y el vial de acero (virutas + el segundo vial vacío).
8. **Bloqueos**: con el chat (T) o el inventario (E) abiertos, ninguna tecla de alomancia (Z, X, R, G, V) actúa.
9. **Guardar y volver**: sal al menú y vuelve a entrar al mundo. La reserva se conserva, los poderes están apagados y no tienes viales ni cartillas de más.

Teclas (reasignables en Opciones → Controles → Alomancia):

| Tecla | Acción |
|---|---|
| V | siguiente pareja de metales (solo hierro/acero tiene cartilla) |
| Z | quemar o apagar el primer metal (hierro) |
| X | quemar o apagar el segundo metal (acero) |
| R | avivar o dejar de avivar el metal que arde |
| G | apagar todo |

## Qué evaluar

| Aspecto | Pregunta | Nota (1–5) y comentario |
|---|---|---|
| Claridad del control | ¿Supiste qué tecla hacía qué sin mirar esta guía? ¿Las teclas chocan con otras? | |
| Respuesta del movimiento | ¿El tirón se siente inmediato, controlable y se detiene bien en el bloque? ¿El avivado se nota? | |
| Utilidad del HUD | ¿El panel deja ver qué metal arde y cuánto queda? ¿Tapa algo? | |
| Recorrido | ¿Fabricar el vial y la cartilla fue claro? ¿Algún mensaje confuso? | |
| Fallos | Cualquier cosa rara (duplicados, poderes encendidos al volver, crash) | |

Si algo falla, anota la hora aproximada: el log del juego está en la carpeta de la instancia (`logs/latest.log`, líneas `MISTBRN`).
