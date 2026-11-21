# Masamune's MISTBRN — diseño

Mod de alomancia para Horizon. Nombre visible: **Masamune's MISTBRN**; id del mod: `mistbrn`.

Este documento describe la experiencia que MISTBRN quiere construir. Los parámetros y el reparto de hitos se ajustarán con pruebas.

## La experiencia

El recorrido comienza en el mundo. El jugador encuentra metales, prepara viales y decide qué reservas llevar. Los poderes deben hacer que una veta, un anclaje o una criatura cambien las opciones disponibles al explorar, combatir o moverse. Practicar una técnica útil y descubrir por qué funciona abre formas nuevas de usarla.

La primera demo se concentra en hierro y acero. El objetivo es preparar un vial, beberlo, atravesar un tramo usando tirones o empujones, consultar la reserva y continuar después de guardar. La versión completa extiende ese recorrido a cuatro parejas de metales, con un diario de investigación y un árbol general de eficiencia.

## Base implementada

P3a ya permite extraer muestras en mundos existentes, fabricar viales para los ocho metales, beberlos y persistir las reservas. Las pruebas en Horizon cubren el ciclo de fabricación y consumo, el rollback de inventario y guardado y el aislamiento entre mundos. Los poderes utilizables, los árboles, el diario y el HUD corresponden a fases posteriores. [Estado actual](STATUS.md) · [Evidencia de P3a](evidence/p3.md)

## Requisitos expresados por el usuario

- Mod para Horizon (playhorizon.dev), inspirado en la alomancia de Mistborn.
- Poderes y recursos necesarios para utilizarlos; desbloqueo de parejas mediante objetos ligados a exploración, dimensiones y jefes.
- Un árbol por pareja, subnodos que potencian cada metal y un árbol general de eficiencia y administración.
- Progresión por práctica e investigación; descubrimientos libres con pistas voluntarias.
- Sol orquesta y delega en modelos adecuados. Checkpoints recuperables incluso al interrumpirse una acción por límites de sesión o tokens.

## Decisiones propuestas para poder ejecutar

Multiplataforma: el mod debe funcionar en todo sistema operativo que soporte el SDK fijado de Horizon (macOS arm64, Windows x64, Linux x64); desarrollo y pruebas primero en macOS arm64, verificación Windows x64 y Linux x64 antes de publicar. Supervivencia individual primero; cooperativo después de verificar soporte autoritativo del servidor. No prometer compatibilidad en servidores vanilla con estos poderes. Ocho metales iniciales. Diario personal con notas encontradas como pistas; esta elección narrativa puede cambiar sin alterar el motor.

La demo V1 es una validación interna de hierro y acero. La primera entrega vertical añade combustible, una investigación, un árbol y guardado (P5). Para los desbloqueos, se propone un tutorial para hierro y acero, un bastión para estaño y peltre, un guardián anciano para zinc y latón y un desafío en una ciudad antigua para cobre y bronce. La dragona habilitaría la maestría final; este reparto es una propuesta de balance, no una decisión previa del usuario. No exige matar al Warden. Todo hito se reclama una sola vez por jugador; duplicados no desbloquean más parejas.

## Arquitectura

El núcleo Rust conserva las reglas y el estado del jugador. El adaptador observa el juego, aplica las transacciones en el servidor y conecta las reglas con Horizon. Los ítems, las recetas y los recursos visuales se declaran en el paquete del mod.

Si el SDK fijado no expone un hook esencial, se documenta la carencia y se presentan opciones al usuario. Un fork de Horizon solo se adopta con su aprobación explícita.

## Mecánicas

Hierro/acero: atraer/empujar metales, monedas, anclajes, movimiento, frenado y desvío de proyectiles metálicos. Fuerzas respecto al jugador, límites de velocidad y colisiones; no vuelo libre ni extracción de bloques.

Estaño/peltre: visión y señales sonoras; fuerza, resistencia, movilidad y avivado con fatiga. No visión de minerales tras paredes ni regeneración automática.

Zinc/latón: provocar, infundir valor, apaciguar y retirada frente a criaturas compatibles; resistencias y recuperación. No controlar acciones de jugadores ni bloquear permanentemente a jefes.

Cobre/bronce: ocultar/detectar emisiones alománticas, distinguir metal y localizar aproximadamente fuentes. Añadir un guardián alomántico y un mecanismo emisor para utilidad individual. La nube propia dificulta detectar. No radar universal.

Cada árbol: dos ramas de tres técnicas, cada técnica con dos subnodos de tres rangos, y una técnica combinada final. Potencia, alcance y precisión son mejoras locales. Las combinadas son encadenamiento, reflejos con esquiva manual, influencia selectiva y rastreo discreto, respectivamente. Deben tener requisitos explícitos, sin ciclos ni dependencias imposibles.

Árbol general: consumo, capacidad, coste de avivado, combustión simultánea, transición y dosificación. No duplicar duración y reducción de consumo para el mismo efecto. Valores iniciales de ensayo: reducción global máxima 20%; capacidad máxima +50%; avivado x2 potencia y x3 consumo, con topes propios por poder. No son valores de balance definitivo.

Práctica otorga experiencia a la pareja por acciones útiles verificadas. Investigación otorga experiencia de pareja y conocimiento general una sola vez por descubrimiento. Los puntos se compran con umbrales definidos en datos; los puntos de pareja no se transfieren entre parejas. Los descubrimientos abren técnicas, los puntos compran su dominio. Experiencia de práctica: rendimientos decrecientes por categoría/objetivo y ventana de tiempo persistida; no recompensar simple combustión ni bucles contra objetos propios sin progreso.

Investigación: indicio → evidencias → conclusión en mesa. Aceptar varias maneras de demostrar un principio. Registrar evidencias válidas incluso antes de encontrar su pista. Tres pistas voluntarias, sin penalización. Nunca exigir una técnica aún bloqueada para descubrir esa misma técnica.

## Recursos e interfaz

La cadena actual usa hierro, cobre, estaño y zinc, además de las aleaciones acero, peltre, latón y bronce. Las muestras se obtienen al minar menas del juego base y están disponibles en chunks ya generados. P3b propone monedas, una capa de bruma y accesorios con arte propio. El cuaderno, la mesa de investigación y los fragmentos ligados a hitos forman parte del recorrido de progresión. Los recursos gráficos son originales y provisionales; antes de publicar se sustituyen por los finales y se registra la licencia de cada recurso.

Libro abre árboles, pistas y reservas. V selecciona pareja; Z/X activa cada metal; R modifica para avivar; G apaga todo. Reasignables; HUD indica reservas y combustiones que continúan al cambiar pareja. Menús/chat capturan entrada; no disparar poderes al escribir. Reducir destellos y señales visuales configurables.

## Persistencia y límites

Identidad por mundo/jugador; guardar reservas, desbloqueos, experiencia, nodos, descubrimientos, evidencias e hitos reclamados. Guardado versionado y transacciones idempotentes para gastar puntos, beber y reclamar. Al morir conservar conocimiento/nodos; vaciar reservas y aplicar reglas normales a inventario. Al reconectar detener combustiones. No reiniciar progreso por perder el cuaderno.

## Éxito

Un jugador puede obtener metal, fabricar vial, beber, usar poder, descubrir, mejorar, guardar y continuar sin duplicaciones. La versión completa añade las cuatro parejas, hitos y recursos. Cooperativo solo se considera entregado tras probar dos clientes y autoridad del servidor. Las pruebas unitarias aisladas no demuestran integración con Horizon.

## Evidencia y límites técnicos

El proyecto compila un paquete `.hmod` nativo contra el SDK fijado. Antes de cada fase se comprueba en `surface.json` que el pin expone los hooks que necesita la entrega. La validación en macOS arm64 y las pruebas pendientes de otras plataformas se detallan en la documentación de compatibilidad. [Compatibilidad](compatibility.md)
