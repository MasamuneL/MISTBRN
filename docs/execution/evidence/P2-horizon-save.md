# P2 — evidencia backend y cuarentena

Fecha: 2026-11-14 America/Mexico_City. Agente: /root/backend_quarantine.
Estado: implemented/no native verification; entrega para revisión independiente.
Worktree: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN.
Rama observada: codex/alomancia-p2-native. HEAD observado: fed744b9252e14ae6fcd12b314fba61040250fe0.

## Alcance y comprobación disponible

Se leyó primero P2-storage-review.md y P2-native.md, después save.rs completo y SDK pin local. Archivos editados por esta tarea: save.rs, horizon-storage.md y ficha/evidencia P2-horizon-save; ficha con temporales/backups únicos. WIP y parciales previos conservados. APIs NativeTestConfig/CreditIntent intactas; único error público nuevo Quarantined. No lib.rs/manifests/core/guard/SDK, commits, Cargo, build, juego, mocks ni tests de texto. /root conserva Cargo e integración nativa.

Revisión estática: inspector habilitante separado de lectura estructural privada que retiene Tags para readback; toda presencia bloquea uso; login inspecciona antes de retornar por Entry; mutate acting/normal y replay rechazan marca; inspección inicial de crédito fija Fault en error; mutate cierra world_fault; test_world canonicaliza ambas rutas antes de comparar y rechaza IO. Tras rollback no confirmado se retiene preimagen y se fija Fault antes de Host. El helper usa Compound actual validado del UUID correcto, conserva state/ajenas/marca existente exactos; setter nuevo exige true y readback estructural exacto. Sin Mutex sostenido durante Host. Se devuelve error, nunca recibo o Ready. Logs separan rollback y marca confirmada/NO confirmada, durable=false.

SDK leído: horizon-world/src/nbt.rs define Compound=BTreeMap<JavaString,Tag>, Tag::String(JavaString), From<&str> y Tag::get(&str). Se preservan claves UTF-16 sin conversión. git diff --check con --work-tree explícito y escalación terminó exit 0 (avisos EOL de WIP ajeno). save.rs es untracked y git diff ordinario no lo muestra. El delta de entrega de abajo se obtiene por git diff --no-index contra la preimagen exacta preservada por /root en p2-stage1; incluye formato aplicado por /root, además de este fix. Exit 1 de --no-index significa diferencias, no fallo de validación. No se invoca compilador ni se acredita comportamiento Host.

Se congeló save.rs y se entregó hash a /root. El intento de generar metadata con Python terminó porque Python no estaba instalado, sin escribir; se usó Node. Una reconstrucción inicial de preimagen en tmp no coincidió y no se usó como evidencia; se sustituyó por la preimagen exacta preservada por /root. No se alteró source para reconstruirla.

## Hashes SHA256

- Preimagen exacta save.rs: 76DB5E05AA76126DBCB0D998A34119CCC537F25D6DD1A752E792FE8F9C994687
- Fuente de preimagen: C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime/native-builds/p2-stage1/bundle/crates/allomancy-horizon/src/save.rs
- save.rs de entrega: 00043AF15B0516BBA1CD7EB7E30568D95C031C932FF695F7304C3019866E3BD0
- docs/horizon-storage.md: CDD7FCC8D231B62EF610845B0E80AD50672547CB03E7D4C8E3B0FA1C91F38487
- Delta UTF-8 LF de abajo: 4DB750A5B7226B22D8479A241F72C9A948EAFDE60A62DF143CC364226F2FD2D7
- Ficha previa a intención: B046B2CCC563F6760CF8B539E429CDCB3D45CD376A001F1D881FB99F065FD464.

## Límites y ensayos pendientes

Sin resultado propio de compilación o ensayo económico. Bool inline/readback confirma memoria, no fsync. Snapshot/setter/readback imposible deja quarantine_unconfirmed y Fault sólo RAM; no acredita próxima apertura segura ni N7. Raíz mods ausente también produce marca NO confirmada; no se repara desde getter vacío. No .dat live ni ledger secundario/sidecar.

AfterInventory, AfterInline y RollbackInventory son fallos inducidos del adaptador. RollbackInventory retorna antes de CAS inverso; no acredita fallo Host, setter, CAS, IO o writer. Pendientes con SDK/Host real: marca v1/scalar/vacía/futura/extra con/sin state; login existente/replay/seed/consumo/acting; rollback exacto AfterInventory/AfterInline; rama artificial más guardado/reapertura N0; CAS inverso realmente rechazado y preservación de ajenas; setter/readback realmente NO confirmado; lifecycle con world_fault; cortes/writer N6–N8 en fronteras observadas. Guard N0 integrado por otra tarea. Recuperación sólo offline desde snapshot completo coherente con identidad y evidencia preservada; quitar sólo la marca no prueba coherencia.

## Delta de entrega sobre preimagen preservada

```diff
diff --git a/C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime/native-builds/p2-stage1/bundle/crates/allomancy-horizon/src/save.rs "b/C:\\Users\\alanj.VIKINGO\\.codex\\worktrees\\mistbrn-p1\\MISTBRN\\crates\\allomancy-horizon\\src\\save.rs"
index 5cdeca6..025576f 100644
--- a/C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime/native-builds/p2-stage1/bundle/crates/allomancy-horizon/src/save.rs
+++ "b/C:\\Users\\alanj.VIKINGO\\.codex\\worktrees\\mistbrn-p1\\MISTBRN\\crates\\allomancy-horizon\\src\\save.rs"
@@ -15,6 +15,7 @@ use std::{
 };

 const NAMESPACE: &str = "allomancy:state";
+const STORAGE_FAULT: &str = "allomancy:storage_fault";

 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum StorageError {
@@ -25,6 +26,7 @@ pub enum StorageError {
     PlayerIdentity,
     Health,
     ModsRoot,
+    Quarantined,
     Namespace,
     FutureTransport(i32),
     Payload,
@@ -103,6 +105,13 @@ pub struct RawPlayer {

 /// Validate RAW Host snapshot before using a normalized persistent-data getter.
 pub fn inspect_player(raw: &Tag, uuid: u128) -> Result<RawPlayer, StorageError> {
+    let player = inspect_player_structure(raw, uuid)?;
+    reject_quarantine(player.mods.as_ref())?;
+    Ok(player)
+}
+
+// Structural reads retain quarantined Tags for exact marker confirmation.
+fn inspect_player_structure(raw: &Tag, uuid: u128) -> Result<RawPlayer, StorageError> {
     if !matches!(raw, Tag::Compound(_)) {
         return Err(StorageError::Root);
     }
@@ -135,6 +144,13 @@ pub fn inspect_player(raw: &Tag, uuid: u128) -> Result<RawPlayer, StorageError>
     })
 }

+fn reject_quarantine(mods: Option<&Tag>) -> Result<(), StorageError> {
+    if mods.is_some_and(|tag| tag.get(STORAGE_FAULT).is_some()) {
+        return Err(StorageError::Quarantined);
+    }
+    Ok(())
+}
+
 fn merge(mods: Option<&Tag>, namespace: Tag) -> Result<Tag, StorageError> {
     let mut map = match mods {
         None => Compound::new(),
@@ -269,7 +285,10 @@ impl SaveBackend {
         };
         if session.world.is_none() {
             if let Some(identity) = &loaded {
-                eprintln!("ALLOMANCY_SAVE event=world_identity source={:?}", identity.source);
+                eprintln!(
+                    "ALLOMANCY_SAVE event=world_identity source={:?}",
+                    identity.source
+                );
             }
         }
         let world = loaded.map(|identity| identity.world_id);
@@ -283,7 +302,16 @@ impl SaveBackend {

     fn test_world(&self, host: &dyn Host) -> Result<WorldId, StorageError> {
         let config = self.test.as_ref().ok_or(StorageError::TestOnly)?;
-        if host.world_dir().as_ref() != Some(&config.world_root) {
+        let actual = host
+            .world_dir()
+            .ok_or(StorageError::TestOnly)?
+            .canonicalize()
+            .map_err(|_| StorageError::TestOnly)?;
+        let expected = config
+            .world_root
+            .canonicalize()
+            .map_err(|_| StorageError::TestOnly)?;
+        if actual != expected {
             return Err(StorageError::TestOnly);
         }
         let world = self.world(host)?;
@@ -328,6 +356,8 @@ impl SaveBackend {
         eprintln!("ALLOMANCY_SAVE event=login");
         let result = (|| {
             let world = self.world(host)?;
+            // Check even an already loaded Entry before the early return.
+            let raw = raw_player(host, uuid)?;
             {
                 let session = self.session.lock().map_err(|_| StorageError::Fault)?;
                 if session.blocked.contains(&uuid) {
@@ -337,7 +367,6 @@ impl SaveBackend {
                     return Ok(());
                 }
             }
-            let raw = raw_player(host, uuid)?;
             let mut saved = match &raw.namespace {
                 Some(namespace) => decode_namespace(namespace, &world, &player_id(uuid))?,
                 None => SavedPlayer::new(player_id(uuid), world, initial_rules())?,
@@ -421,6 +450,9 @@ impl SaveBackend {
     {
         let (before, namespace, generation) = {
             let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
+            if session.world_fault {
+                return Err(StorageError::Identity);
+            }
             let entry = session
                 .players
                 .get_mut(&uuid)
@@ -443,6 +475,7 @@ impl SaveBackend {
                 let mods = host
                     .player_persistent_data(uuid)
                     .ok_or(StorageError::Unavailable)?;
+                reject_quarantine(Some(&mods))?;
                 if !matches!(mods, Tag::Compound(_)) || mods.get(NAMESPACE) != Some(&namespace) {
                     return Err(StorageError::Conflict);
                 }
@@ -462,7 +495,9 @@ impl SaveBackend {
                 return Err(StorageError::HostRejected);
             }
             let observed = if acting {
-                host.player_persistent_data(uuid)
+                let observed = host.player_persistent_data(uuid);
+                reject_quarantine(observed.as_ref())?;
+                observed
             } else {
                 raw_player(host, uuid)?.mods
             };
@@ -748,7 +783,13 @@ impl SaveBackend {
         preparation: CreditPreparation,
     ) -> Result<CreditReceipt, StorageError> {
         let uuid = intent.player;
-        let raw = raw_player(host, uuid)?;
+        let raw = match raw_player(host, uuid) {
+            Ok(raw) => raw,
+            Err(error) => {
+                self.fault(uuid, &error);
+                return Err(error);
+            }
+        };
         if raw.dead || raw.namespace.as_ref() != Some(namespace) {
             self.fault(uuid, &StorageError::Conflict);
             return Err(StorageError::Conflict);
@@ -863,6 +904,15 @@ impl SaveBackend {
                         }
                     }
                     self.fault(uuid, &StorageError::RollbackFailed);
+                    eprintln!("ALLOMANCY_SAVE event=rollback_unverified");
+                    match self.quarantine(host, uuid) {
+                        Ok(()) => eprintln!(
+                            "ALLOMANCY_SAVE event=quarantine_confirmed accepted_in_memory=true durable=false"
+                        ),
+                        Err(error) => eprintln!(
+                            "ALLOMANCY_SAVE event=quarantine_unconfirmed reason={error:?} durable=false"
+                        ),
+                    }
                     Err(StorageError::RollbackFailed)
                 } else {
                     eprintln!("ALLOMANCY_SAVE event=rollback_verified");
@@ -872,6 +922,47 @@ impl SaveBackend {
         }
     }

+    // Only the current valid raw Compound may be marked; never restore a stale
+    // root or trust the normalized getter to repair missing/invalid data.
+    fn quarantine(&self, host: &mut dyn Host, uuid: u128) -> Result<(), StorageError> {
+        let current = inspect_player_structure(
+            &host
+                .player_compound(uuid)
+                .ok_or(StorageError::Unavailable)?,
+            uuid,
+        )?;
+        let already_marked = current
+            .mods
+            .as_ref()
+            .is_some_and(|tag| tag.get(STORAGE_FAULT).is_some());
+        let Some(Tag::Compound(mut mods)) = current.mods else {
+            return Err(StorageError::ModsRoot);
+        };
+        if !already_marked {
+            mods.insert(
+                STORAGE_FAULT.into(),
+                Tag::compound([
+                    ("format_version", Tag::Int(1)),
+                    ("reason", Tag::String("rollback_unverified".into())),
+                ]),
+            );
+        }
+        let expected = Tag::Compound(mods);
+        if !already_marked && !host.set_player_persistent_data(uuid, expected.clone()) {
+            return Err(StorageError::HostRejected);
+        }
+        let observed = inspect_player_structure(
+            &host
+                .player_compound(uuid)
+                .ok_or(StorageError::Unavailable)?,
+            uuid,
+        )?;
+        if observed.mods != Some(expected) {
+            return Err(StorageError::Conflict);
+        }
+        Ok(())
+    }
+
     fn rollback(
         &self,
         host: &mut dyn Host,
```
