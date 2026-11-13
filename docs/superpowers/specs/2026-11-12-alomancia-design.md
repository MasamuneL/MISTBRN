# Alomancia para Horizon — diseño de partida

Estado: propuesta consolidada para planificar; parámetros ajustables mediante pruebas. Fecha: 2026-11-12.

## Requisitos expresados por el usuario

- Mod para Horizon (playhorizon.dev), inspirado en la alomancia de Mistborn.
- Poderes y recursos necesarios para utilizarlos; desbloqueo de parejas mediante objetos ligados a exploración, dimensiones y jefes.
- Un árbol por pareja, subnodos que potencian cada metal y un árbol general de eficiencia y administración.
- Progresión por práctica e investigación; descubrimientos libres con pistas voluntarias.
- Sol orquesta y delega en modelos adecuados. Checkpoints recuperables incluso al interrumpirse una acción por límites de sesión o tokens.

## Decisiones propuestas para poder ejecutar

Windows x64 y supervivencia individual primero; cooperativo después de verificar soporte autoritativo del servidor. No prometer compatibilidad en servidores vanilla con estos poderes. Ocho metales iniciales. Diario personal con notas encontradas como pistas; esta elección narrativa puede cambiar sin alterar el motor.

Primera entrega vertical: hierro/acero, combustible, una investigación, árbol y guardado. Versión completa: cuatro parejas y árbol general. Un tutorial entrega acceso a hierro/acero; un bastión desbloquea estaño/peltre, un guardián anciano zinc/latón y un desafío de ciudad antigua cobre/bronce. La dragona habilita maestría final; este reparto es una propuesta de balance, no una decisión previa del usuario. No exige matar al Warden. Todo hito se reclama una sola vez por jugador; duplicados no desbloquean más parejas.

## Enfoques considerados

1. Mod nativo Rust con contenido declarativo: recomendado; permite separar lógica y recursos, sujeto a comprobar hooks.
2. Solo contenido declarativo: útil para objetos/recetas, insuficiente si no admite física, poderes e interfaz.
3. Modificar un fork de Horizon: alternativa si faltan hooks esenciales; mayor mantenimiento. No adoptar silenciosamente: documentar carencias y presentar la decisión antes de comprometer el proyecto a un fork.

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

Materias primas: hierro, cobre y zinc/estaño nuevos; aleaciones acero, peltre, latón y bronce. Recetas y obtención en datos. Viales vacíos/llenos, muestras, monedas, cuaderno, mesa y fragmentos ligados a hitos. Añadir fuentes de muestras en mundos ya creados; no obligar a regenerarlos. Recursos gráficos originales provisionales, con sustitución final y registro de licencias.

Libro abre árboles, pistas y reservas. V selecciona pareja; Z/X activa cada metal; R modifica para avivar; G apaga todo. Reasignables; HUD indica reservas y combustiones que continúan al cambiar pareja. Menús/chat capturan entrada; no disparar poderes al escribir. Reducir destellos y señales visuales configurables.

## Persistencia y límites

Identidad por mundo/jugador; guardar reservas, desbloqueos, experiencia, nodos, descubrimientos, evidencias e hitos reclamados. Guardado versionado y transacciones idempotentes para gastar puntos, beber y reclamar. Al morir conservar conocimiento/nodos; vaciar reservas y aplicar reglas normales a inventario. Al reconectar detener combustiones. No reiniciar progreso por perder el cuaderno.

## Éxito

Un jugador puede obtener metal, fabricar vial, beber, usar poder, descubrir, mejorar, guardar y continuar sin duplicaciones. La versión completa añade las cuatro parejas, hitos y recursos. Cooperativo solo se considera entregado tras probar dos clientes y autoridad del servidor. Las pruebas unitarias aisladas no demuestran integración con Horizon.

## Evidencia y límites técnicos

El README oficial consultado describe .hmod para contenido y .hnmod para Rust: https://github.com/horizon-rust/horizon#mods-and-shader-packs . La existencia de esos formatos no confirma cada hook requerido. La fase P0 debe verificar el código y fijar commit/versión antes de definir el adaptador. No existe código del mod en este proyecto al redactar este diseño.
