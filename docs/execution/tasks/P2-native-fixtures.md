# P2 generador offline de fixtures nativos

tools/make-native-fixture.cjs utiliza solo Node estándar. Sintaxis comprobada con node --check; no se ejecutaron fixtures, mundos ni Cargo. Antes de usar, confirmar por separado que Horizon está cerrado y que el mundo es una copia dedicada, no datos personales. Cada kind requiere copia independiente con marker existente y ruta canónica runtime/game-p2-*/saves/NAME.

```powershell
node tools/make-native-fixture.cjs --world '<runtime>/game-p2-case/saves/N0' --kind fault
```

Crea players/data si falta y exclusivamente el UUID 00000000-0000-0000-0000-000000000001.dat; rechaza si ya existe .dat o .dat_old, sin distinguir mayúsculas. dat-uppercase usa .DAT. backup-fault crea primary foreign y backup con marca usando wx en cada archivo: si falla la segunda escritura, conservar e inspeccionar el primary creado; no reutilizar la copia sin revisión. El stdout JSON lista tipo, expectativa, paths, bytes y SHA256.

Kinds: foreign conserva foreign:keep Int7 y espera accept; todos los demás esperan reject: fault y fault-without-state contienen marca canónica format_version Int1/reason rollback_unverified sin state; fault-scalar marca Byte0; fault-future Compound versión 2; future-transport state versión 2; future-json payload {"schema_version":999}; corrupt-json payload truncado; invalid-root NeoForgeData Int; duplicate repite una clave NBT en raíz; truncated corta gzip a diez bytes; dat-uppercase repite duplicate en extensión mayúscula; backup-fault marca solo en backup. Cada root lleva UUID IntArray [0,0,0,1] y Health Float20 salvo que gzip truncado impide leerlo. Cada caso conserva otros archivos y marcador.

Estos archivos extra no son una demostración de jugador jugable ni de N7, rollback real, durabilidad o recuperación. future-json es documento mínimo futuro, no transformación de un save válido; confirma rechazo de versión si la traza específica lo identifica. La expectativa debe acreditarse ejecutando guard integrado y verificando hashes/logs con el harness, no se considera aprobada por generar bytes. El script no comprueba procesos ni protege contra sustitución concurrente de rutas: mantener exclusividad operativa.
