use allomancy_core::*;

// Catches nodes the map or the server would not find, codes past op 6, and a bridge in the wrong
// pool or needing the wrong branches.
#[test]
fn the_zinc_brass_catalog_holds() {
    assert_eq!(validate_catalog(&CATALOG), Ok(()));
    for code in [
        NODE_ZINC_RANGE,
        NODE_ZINC_DURATION,
        NODE_COURAGE,
        NODE_COURAGE_LEVEL,
        NODE_COURAGE_DURATION,
        NODE_BRASS_RANGE,
        NODE_BRASS_DURATION,
        NODE_RETREAT,
        NODE_RETREAT_DISTANCE,
        NODE_RETREAT_SPEED,
        NODE_SELECTIVE,
    ] {
        let def = node_def(code, &CATALOG).unwrap();
        assert!(code >= 64, "{code} is bought with op 9");
        assert_eq!(def.pool, Pool::Pair(Pair::ZincBrass), "{code}");
    }
    let bridge = node_def(NODE_OUTER_REACH, &CATALOG).unwrap();
    assert_eq!(bridge.pool, Pool::Knowledge);
    assert_eq!(
        bridge.requires,
        &[
            Requirement::Node {
                code: NODE_REACH,
                rank: 1
            },
            Requirement::Node {
                code: NODE_BRASS_RANGE,
                rank: 1
            },
        ]
    );
    let selective = node_def(NODE_SELECTIVE, &CATALOG).unwrap();
    assert!(selective.requires.contains(&Requirement::Discovery {
        code: RESEARCH_EMOTION
    }));
    for category in [PRACTICE_PROVOKE, PRACTICE_SOOTHE, PRACTICE_ALLY] {
        assert_eq!(
            practice_def(category, &CATALOG).unwrap().pair,
            Pair::ZincBrass
        );
    }
    let research = research_def(RESEARCH_EMOTION, &CATALOG).unwrap();
    assert_eq!(
        (research.pair, research.needed, research.evidence),
        (Pair::ZincBrass, 2, 7)
    );
}
