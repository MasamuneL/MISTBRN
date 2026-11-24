# Guía P5: práctica, investigación y cuaderno

Recorrido a mano de la primera entrega de P5. Apunta lo que notes en la tabla del final.

## Preparación

- Instancia `mistbrn-dev` con el build de la PR (`tools/build-native.sh`).
- Mundo nuevo en **Supervivencia**, con trucos. Sin `MISTBRN_TEST_SCENARIO`.
- Materiales (por chat): `/give @s glass 2`, `/give @s iron_ingot 4`, `/give @s cobblestone 2`, `/give @s coal 2`, `/give @s book 1`, `/give @s iron_nugget 1`, `/give @s iron_block 2`, `/give @s crafting_table 1`.

## Recorrido

1. **Vial de hierro** (cuadrícula 2×2): 2 cristales → 2 viales vacíos; lingote de hierro + adoquín → virutas; vial vacío + virutas → vial de hierro.
2. **Tutorial**: bebe el vial (mantén clic derecho). El chat dice «Alomancia: el metal arde en tu interior. Ya puedes quemar hierro (Z) y acero (X).» y aparecen las barras de hierro (25 %) y acero (vacía) encima de la de hambre. Ya no hay cartilla: el primer vial de hierro o de acero enseña la pareja.
3. **Cuaderno**: libro + pepita de hierro. Clic derecho al aire con él, o tecla **J** (también sin el ítem; reasignable), abre la pantalla. **Esc** o **J** la cierran.
4. **Práctica**: coloca un bloque de hierro a unos 10 bloques, míralo y pulsa **Z**: el tirón te arrastra. Apaga con **Z** o **G**. Al arrastrarte, el chat dice «Cuaderno: anotaste algo nuevo en el diario.»: es una evidencia del paso 5. Cada tirón que te mueve 2 bloques o más da experiencia a hierro y acero (10, luego 5, luego 2 en el mismo minuto; después nada hasta que pase el minuto). Quemar sin moverte no da nada. La experiencia sale en **Árbol**.
5. **Investigación** (pestaña **Diario**): «El principio del ancla» pide 2 de 3 evidencias, en el orden que quieras:
   - atraer con hierro un metal suelto (suelta un lingote con Q, aléjate, míralo y pulsa Z) hasta recogerlo;
   - que un bloque de hierro fijo te arrastre (paso 4);
   - empujar con acero un bloque de hierro bajo tus pies y elevarte 1,3 bloques o más: lingote de hierro + 2 carbones → lingote de acero; acero + adoquín → virutas de acero; virutas + el otro vial vacío → vial de acero. Bébelo, súbete al bloque, mira abajo y pulsa **X**.
   Las pistas son voluntarias: el botón «Ver pista 1/3» muestra una cada vez, sin penalización. Las evidencias cuentan aunque no hayas visto ninguna pista.
6. **Concluir**: con 2 evidencias el chat avisa «Cuaderno: ya puedes sacar una conclusión…» y el Diario dice «lista». Coloca la mesa y haz clic derecho sobre ella con el cuaderno en la mano: «Descubrimiento: el principio del ancla…» (+200 de experiencia de pareja y 3 puntos de conocimiento; la mesa no se abre). Repetirlo dice «Ya descubriste todo lo que tus notas permiten.» y no da nada.
7. **Comprar**: en **Árbol**, pasa el ratón sobre un nodo para ver efecto, coste y requisitos; clic para mejorar si está disponible (marco con brillo de latón). El chat confirma «Cuaderno: técnica mejorada.». Prueba **Alcance** (1 punto de pareja): pon el bloque a 17 bloques; sin Alcance el tirón no llega, con Alcance 1 sí (18). Prueba **Capacidad** (1 de conocimiento): caben 110 000 en vez de 100 000, así que un vial llena menos barra. **Salto con moneda** ya no pide el descubrimiento, pero cuesta 2 puntos de pareja: con unos 217 de experiencia tienes 1, así que dirá «Te faltan puntos.».
8. **Persistencia**: guarda y sal, vuelve a entrar; muere (`/kill @s`) y reaparece. Experiencia, nodos, evidencias y pistas se conservan; las reservas se vacían al morir, como siempre.

Un mundo de V1 se migra solo: conserva desbloqueos y reservas, con progreso vacío. Las cartillas viejas funcionan, pero ya no se fabrican.

Valores de ensayo (puntos, efectos y costes): `docs/plan.md`, sección P5.

## Tu evaluación

| Aspecto | Nota |
|---|---|
| ¿Se entiende el tutorial del vial? | |
| ¿La práctica da experiencia a buen ritmo? | |
| ¿Se entienden las evidencias y las pistas? | |
| ¿Es claro el cuaderno (árbol, compra, diario)? | |
| ¿Se notan alcance y capacidad al comprarlos? | |
