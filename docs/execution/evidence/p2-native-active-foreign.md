# Namespace ajeno del jugador activo

Build p2-owner1 sin cambios. Se prepararon dos copias cerradas del baseline AllomancyQA y se insertó exclusivamente `NeoForgeData[foreign:keep]=Int7` en el archivo del propietario real y su backup, preservando los originales y los bytes del payload. Es una fixture offline, jamás una escritura manual de DAT vivo.

owner-foreign-credit1 (roundtrip) y owner-foreign-quarantine1 (fallo inverso artificial del adaptador) pasaron Host real, quit y WorldCloseFinal/exit0. El inspector adjunto verifica cuatro snapshots DAT/old: clave extranjera con nombre, tipo Int y valor7 intactos; identidadB y economía esperada. Crédito: reserva1000, ledger1, inventario2. Cuarentena: reserva0, ledger0, inventario2 y marca canónica de rollback no verificado.

Esto cubre merge del jugador activo, adicional al N0 anterior que sólo preservaba un archivo ajeno. No acredita un error espontáneo del Host ni fsync. Los fixtures, receipts y originales se conservan en game-p2-active-foreign y native-credit-runs. El JSON adjunto incluye hashes de cada archivo.