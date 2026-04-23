use ishou_tokens::TokenSet;

#[test]
fn default_token_set_is_stable() {
    let tokens = TokenSet::pleme();
    insta::assert_json_snapshot!(tokens);
}

#[test]
fn content_hash_is_deterministic() {
    let a = TokenSet::pleme().content_hash();
    let b = TokenSet::pleme().content_hash();
    assert_eq!(a, b);
}

#[test]
fn content_hash_changes_on_token_change() {
    let mut a = TokenSet::pleme();
    a.spacing.px_4 = 20; // was 16
    let b = TokenSet::pleme();
    assert_ne!(a.content_hash(), b.content_hash());
}

#[test]
fn every_semantic_role_resolves_to_a_palette_entry() {
    let tokens = TokenSet::pleme();
    for (role, key) in tokens.roles.pairs() {
        assert!(
            tokens.color.get(key).is_some(),
            "role {role} → {key} not found in palette"
        );
    }
}
