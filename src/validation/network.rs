use super::*;

/// Tokens that mark a non-production chain identity.
const NON_MAINNET_TOKENS: [&str; 6] = ["test", "dev", "local", "rehears", "fixture", "synthetic"];

pub(super) fn check(i: &ContractInput) -> Check {
    let c = &i.chain;
    // CHOICE: real migrated state only on Live chains, but not only on mainnet.
    // The Χ rung rehearses real data on a testnet-labelled Live chain, so the
    // label is free for migration-input while Development/Local are refused.
    // Conversely a synthetic fixture can never carry the mainnet label.
    if i.purpose == Purpose::MigrationInput && c.chain_type != ChainType::Live {
        return Err(fail(
            "chain_purpose_binding",
            "/chain/chainType",
            "migration-input carries real state and requires chainType Live",
        ));
    }
    if c.network == Network::Mainnet && i.purpose != Purpose::MigrationInput {
        return Err(fail(
            "chain_purpose_binding",
            "/chain/network",
            "only migration-input may be labelled mainnet",
        ));
    }
    if c.id.is_empty()
        || c.id.len() > 64
        || !c
            .id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err(fail(
            "chain_identity_label",
            "/chain/id",
            "chain id must be 1-64 characters of [a-z0-9_]",
        ));
    }
    if c.name.is_empty() || c.name.trim() != c.name || c.name.chars().any(char::is_control) {
        return Err(fail(
            "chain_identity_label",
            "/chain/name",
            "chain name must be nonempty without surrounding whitespace or control characters",
        ));
    }
    // CHOICE: label consistency by token, not an exact id pin. No mainnet chain id
    // has been ruled yet; this refuses the mislabels S-6 describes (a testnet id
    // shipped as mainnet, or a rehearsal chain named like mainnet) without inventing one.
    let name = c.name.to_ascii_lowercase();
    match c.network {
        Network::Mainnet => {
            for (field, value) in [("id", c.id.as_str()), ("name", name.as_str())] {
                if let Some(token) = NON_MAINNET_TOKENS.iter().find(|t| value.contains(*t)) {
                    return Err(fail(
                        "chain_identity_label",
                        format!("/chain/{field}"),
                        format!("mainnet identity cannot contain non-production token `{token}`"),
                    ));
                }
            }
        }
        Network::Testnet => {
            for (field, value) in [("id", c.id.as_str()), ("name", name.as_str())] {
                if !value.contains("test") {
                    return Err(fail(
                        "chain_identity_label",
                        format!("/chain/{field}"),
                        "testnet identity must contain `test`",
                    ));
                }
            }
        }
    }
    Ok(())
}
