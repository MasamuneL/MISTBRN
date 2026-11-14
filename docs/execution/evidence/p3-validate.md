# Evidencia P3: validación interna

- `pwsh -NoProfile -File tools/test-content-validation.ps1`: RED con stub: baseline pasa, nueve negativos fallan porque se aceptan (exit 1; p3-validation-red.log). GREEN: baseline y nueve negativos pasan (exit 0; p3-validation-green.log). Fixtures y resultados preservados bajo `target/content-validation-tests/9117954e18424e34b11fe15d4a005333`.
- `pwsh -NoProfile -File tools/validate-content.ps1`: exit 0, 8 metales, 4 parejas, 32 items, 4 hitos, 26 recetas, 7 fuentes y 2 trades; 26 recetas alcanzables desde fuentes declaradas.
- Parser AST PowerShell: ambos scripts, cero errores.
- La alcanzabilidad incluye fuentes propuestas sin verificar; es un escenario declarado, no evidencia de disponibilidad real, compatibilidad SDK ni integración Horizon. Los campos propuestos se preservaron. Sin tests/builds Horizon ni poderes.


## Ronda 1 de revisión
RED: `pwsh -NoProfile -File tools/test-content-validation.ps1` contra el validador anterior: seis casos nuevos aceptados (exit 1 de suite), log `target/p3-fix1-red.log`. GREEN: misma orden exit 0, 16 casos pasan, log `target/p3-fix1-green.log`, fixtures `target/content-validation-tests/3cd43178fbdf4e78bddf195de9cd5252`. Se exige tipo bool en cuatro campos, se rechaza grant desconocido y se verifica DAG independientemente del cierre. El caso previo `cycle` sigue comprobando inalcanzabilidad. Alcance únicamente interno, sin SDK/Horizon.


## Ronda 2 de revisión
`pwsh -NoProfile -File tools/test-content-validation.ps1`: RED exit 1 con `grant_true` y `grant_null` aceptados; los otros tres nuevos negativos ya eran rechazados. GREEN exit 0, 21 casos, incluidos los 16 anteriores. Logs `target/p3-fix2-red.log` y `target/p3-fix2-green.log`; fixtures finales `target/content-validation-tests/3fe0f156c33240bbac537e465378592a`. `grants`, si existe, debe ser cadena exactamente igual al símbolo permitido; ausencia sigue permitida. Sin integración SDK/Horizon.

## Normalización precommit
Solo se quitó la línea vacía final extra de content-catalog.md, P3-validate.md y ambos scripts, y dos espacios finales en p3-catalog.md. Comparación exacta con las copias previas y `git diff --check` del alcance: sin errores. AST: cero errores en ambos scripts. La suite 21/21 es la aprobada antes de este cambio sin lógica; no se repitió ni se atribuye revisión nueva.

SHA256: content-catalog.md B602CB6BF4D5AC7A6193F580B823FE3161914D4969744501FE5E58E0520742C6; tasks/P3-validate.md CFC6A27714090139973948C178370BB9E3BA0E06FE7829BB146D29A5C1455409; test-content-validation.ps1 41BD20958592809B6711AF4256E98D3CAFC73C99D5BDC1686467F96C27F75FA3; validate-content.ps1 15E05B1EF28B5E0927F0357C586330939ABFDBB8BEA54201F66145CFC2D1CD8F; p3-catalog.md 46BD4772DF53C747A7A4E33D74B74B72959B26E47E260409BA75D28FD202E9AD.
