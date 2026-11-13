# P1-tooling — preparación independiente
schema_version:1
fecha:2026-11-13 America/Mexico_City
estado:in_progress (herramientas; P1 código espera C0)
owner:principal
base:6e5f001d / codex/alomancia-p0
Intención: añadir rustfmt y clippy1.99.1 oficiales solo a la toolchain portable del proyecto; se verificó que no están en bin. No instalar Rust global ni modificar configuración del usuario.
Destino: experiments/horizon-probe/runtime/launcher/rust/1.99.1-x86_64-pc-windows-gnu/bin
Paquetes y logs: runtime/devtools; docs/execution/evidence/p1-tooling.log (local)
Proveniencia: static.rust-lang.org/dist/<component>-1.99.1-x86_64-pc-windows-gnu.tar.xz; verificar .sha256 antes de extraer; rechazar rutas fuera de paquete.
Recuperación: revisar archivo/hash y existencia/versiones de binarios antes de repetir; no reinstalar toolchain base. Próximo: descargar/verificar/copiar componentes ausentes y ejecutar --version.

Primera descarga rustfmt+sha verificada,guarddearchivo rechazó entrada raíz exacta sin slash;tar-tf confirmó paquete convencional y no se extrajo/copió. Próximo: aceptar solo raíz exacta ademásde descendientes,conservarrechazo .. / drive; reusar archivohashverificado y continuar componentes.
Resultado: amboscomponentes instaladossoloenscratch;sha256rustfmtA578F900285F6537D4332608E51C9591E2AA6A82DA7ED0D079D2EE9530CD3F01;clippy36C0066126EA9462412DD52DEAC1C603E645812A8B251FB255BEEC61A31000EF. rustfmt1.10.0/clippy0.1.99 mismoRustcommit58cf8f9c35,versionesexit0. Estadoverified;próximoP1usarbinportable explícito,sin reinstalar.
Incidencianueva: rustdoc.exe figurabainventarioprevioahoranoexiste,confirmadoRootySol;ningúnagenteautorizóaeliminarlo. 13testsCoreGREENperoDOCexit101/nunca ejecutado. Causadesapariciónnoestablecida;noatribuiraseguridadsinprueba. Intención: restaurarSOLO rustdoc.exe delarchivorustc1.99.1oficial trasSHAverificado,no reinstalartoolchainnihacerajustesOS;verificarversionypersistenciadelarchivoantesdedoctests. Si vuelveadesaparecer,no buclereinstalar.
RestauraciónúnicarustcpackageSHA256A48F186DEFB3D57538B34E14A24A94F6E29906F191630B627DCB091198BAE438,entrada rustdoc21MBextraídaoficial;trascopiarysolicitarversionarchivoOTRAVEZausente,no salida deversion capturada. Norepetirrestauración. Causaexterna noestablecidaaún;consultaeventosreadonlysecurityrustdoc_count=0,sin alterarlos. Corepruebas13validas;docgatebloqueadotooling.