# Rapport de tests unitaires — industrial-protocols-rust

- **Date** : 2026-08-27
- **Version** : 1.1.2 (workspace)
- **Conclusion** : ✅ Tout passe — 926 tests réussis, 0 échec, 3 ignorés (gated par matériel)

## Résumé

| Élément | Résultat |
|---|---|
| `cargo test --workspace` | ✅ 926 réussis / 0 échec / 3 ignorés |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 avertissements |
| `cargo fmt --all --check` | ✅ Réussi |
| Crates couverts | 46 / 46 |
| Nouvelles fonctions de test ajoutées | environ 727 |

## Nombre de tests par crate

| crate | Nombre de tests | crate | Nombre de tests |
|---|---|---|---|
| kernel | 123 | protocols/kline | 24 |
| async-layer | 9 | protocols/lightbus | 26 |
| bridge/can | 5 | protocols/lin | 24 |
| bridge/ffi | 5 | protocols/lonworks | 22 |
| bridge/serial | 9 | protocols/modbus | 36 |
| bridge/socket | 9 | protocols/modbusplus | 22 |
| bridge/vendor | 6 | protocols/most | 21 |
| protocols/asinterface | 20 | protocols/mqtt | 23 |
| protocols/bacnet | 20 | protocols/opcua | 13 |
| protocols/canopen | 25 | protocols/pci | 22 |
| protocols/cclink | 20 | protocols/powerlink | 20 |
| protocols/cclinkie | 18 | protocols/profibus | 22 |
| protocols/controlnet | 18 | protocols/profinet | 16 |
| protocols/cpci | 19 | protocols/saej1850 | 15 |
| protocols/dali | 24 | protocols/sercos | 17 |
| protocols/devicenet | 21 | protocols/vme | 22 |
| protocols/dnp3 | 15 | protocols/wirelesshart | 18 |
| protocols/ethercat | 12 | protocols/worldfip | 19 |
| protocols/ethernet-ip | 16 | protocols/hartip | 18 |
| protocols/flexray | 20 | protocols/iec61850 | 18 |
| protocols/foundationfieldbus | 12 | protocols/interbus | 22 |
| protocols/hart | 14 | protocols/iolink | 24 |
| protocols/isa100 | 23 | | |

## Couverture

- **Construction de trames** : séquences d'octets exactes, longueurs, boutisme (endianness), sommes de contrôle (CRC-16/LRC/XOR vérifiées indépendamment et en croisement)
- **Analyseurs** : chemin nominal + cas limites (entrées tronquées/courtes, offsets hors limites) (tous renvoient une erreur, sans panic)
- **Métadonnées** : name()/version()/category()/supported_variants()
- **ProtocolRead/Write** : read/read_batch/read_range via une `Connection` mock (réponses prédéfinies + modes de défaillance `io::Error` + enregistrement des trames envoyées)
- **ProtocolHealth** : chemins healthy / degraded / unhealthy
- **ProtocolDiagnostics** : intégrité des champs
- **Noyau** : transition half-open du disjoncteur, limite de nouvelles tentatives, valeurs par défaut de la configuration, filtrage des événements, pool de connexions, compteurs d'erreurs du moteur de passerelle
- **Ponts** : loopback réel socket, refus d'écriture après fermeture, loopback UDP ; chemins d'erreur serial/can/ffi

## Défauts corrigés (découverts par ces tests)

| Emplacement | Défaut | Correctif |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` calcule le bit de parité P1 avec la négation bit à bit `!x`, qui force toujours 0x80, rendant tous les PID LIN invalides | Remplacé par la négation logique `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` panique (tranche hors limites) sur les trames malformées avec len<2 | Remplacé par `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` panique sur entrée tronquée (longueur/sujet/ID de paquet) | Ajout de 3 vérifications de bornes |
| protocols/opcua/src/lib.rs | le champ MessageSize de `build_read_request` sous-comptait 4 octets | Remplacé par `8 + 4 + body.len()` |
| kernel/src/config.rs | en l'absence de table `[defaults]`, désérialisation à zéro au lieu des valeurs par défaut documentées | Ajout de la fonction serde `default_defaults()` |
| bridge/socket/src/lib.rs | après `close()`, `is_open()` renvoyait encore true | Ajout du drapeau `closed` |

## Reproduction

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
