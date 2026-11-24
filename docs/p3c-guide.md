# P3c — Guía para probar la alquimia

Los viales de metal ya no se fabrican en la mesa: se destilan en el **alambique** en dos pasos. La receta vieja (vial vacío + viruta) desaparece, pero los viales que ya tengas se siguen bebiendo igual.

| Qué | Cómo se consigue |
|---|---|
| Vial de agua | Clic derecho con un vial vacío mirando agua (una fuente). El agua no se gasta. |
| Sal | Piedra de sal (en las playas) da de 1 a 5; con Toque de Seda da el bloque. La arena de playa que rompes da sal 1 de cada 20 veces. En el horno, un vial de agua da 1 de sal y el vial vacío sale encima del horno. |
| Alambique | Mesa de trabajo: vidrio, hueco, vidrio arriba; cobre, caldero, cobre abajo. |
| Solución salina | Alambique: viales de agua + 1 de sal. |
| Vial de metal | Alambique: soluciones salinas + 1 viruta de ese metal. |
| Vial de metales básicos | Mesa: un vial de cada uno de los 8 metales + polvo de blaze. |

**El alambique** tiene 3 huecos de vial (uno por hueco) y uno de ingrediente. No usa combustible. Cada tanda dura 30 s (600 ticks), convierte todos los viales que estén en la etapa del ingrediente y gasta 1 ingrediente: una viruta da 3 viales. La barra junto al ingrediente muestra el avance, y al pasar el ratón por ella ves cuánto falta. La tanda sigue tras guardar y salir. Un ingrediente de la otra etapa (sal con soluciones, viruta con agua) no hace nada.

**El vial de metales básicos** suma 25 000 a la reserva de cada metal **desbloqueado**, sin pasar de su capacidad. No desbloquea nada. Si ninguna reserva desbloqueada tiene sitio, no se gasta y sale un aviso en el chat.

## Prueba rápida en creativo (con trucos)

Usa un mundo nuevo.

1. **Agua.** Coloca agua con un cubo. Ponte en supervivencia (`/gamemode survival`), coge un vial vacío y haz clic derecho mirando el agua: tienes un vial de agua. Mirando piedra no pasa nada.
2. **Sal.** Escribe `/setblock ~2 ~ ~ mistbrn:salt_rock` y rómpelo con un pico: caen de 1 a 5 de sal. Con un pico con Toque de Seda cae el bloque. Pon un vial de agua y carbón en un horno: sale sal, y el vial vacío cae encima del horno.
3. **Alambique.** Fabrícalo o escribe `/give @s mistbrn:alembic`, colócalo y ábrelo con la mano vacía. Pon 3 viales de agua y 1 de sal. A los 30 s tienes 3 soluciones salinas. Pon una viruta de hierro: a los 30 s tienes 3 viales de hierro.
4. **Guardado.** Empieza una tanda, sal del mundo a mitad y vuelve a entrar: termina cuando le tocaba, no vuelve a empezar desde cero.
5. **Básicos.** En un mundo nuevo, bebe un vial de metales básicos: sale el aviso y no se gasta. Bebe uno de hierro y luego el de básicos: suben el hierro y el acero, y nada más.
6. **Playa.** Escribe `/locate biome minecraft:beach` y ve allí: en la arena hay piedras de sal sueltas, solo en terreno que no se había generado antes.

En supervivencia es igual; solo cuesta reunir los materiales.

## Qué anotar

- Si una tanda no termina, termina antes de 30 s o gasta más de un ingrediente.
- Si algo se pierde o se duplica al sacar viales a mitad de tanda o al romper el alambique.
- Si la pantalla del alambique se ve mal (huecos, barra, textos).
- Si la piedra de sal sale fuera de la arena de playa o en exceso.
- Si un mundo de antes pierde viales, reservas o progreso.

Envía las notas en el chat; se añaden a `docs/evidence/p3c.md`.
