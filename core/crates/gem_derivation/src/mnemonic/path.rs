pub(super) const HARDENED_OFFSET: u32 = 1 << 31;

#[derive(Clone, Copy)]
pub(super) struct DerivationPathComponent {
    pub(super) index: u32,
    pub(super) hardened: bool,
}

pub(super) fn parse_derivation_path(path: &str) -> Option<Vec<DerivationPathComponent>> {
    let mut parts = path.split('/');
    if parts.next() != Some("m") {
        return None;
    }

    let mut components = Vec::new();
    for part in parts {
        if part.is_empty() {
            return None;
        }
        let (raw_index, hardened) = if let Some(raw_index) = part.strip_suffix('\'') {
            (raw_index, true)
        } else if let Some(raw_index) = part.strip_suffix('h') {
            (raw_index, true)
        } else {
            (part, false)
        };
        let index = raw_index.parse::<u32>().ok()?;
        if index >= HARDENED_OFFSET {
            return None;
        }
        components.push(DerivationPathComponent { index, hardened });
    }
    Some(components)
}

pub(super) fn component_number(component: DerivationPathComponent) -> u32 {
    if component.hardened { component.index | HARDENED_OFFSET } else { component.index }
}
