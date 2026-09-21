use std::collections::HashMap;

use primitives::{GEM_API_HOST, node_config::NodeRegion};

use super::GemLatencyStatus;
use crate::models::list::{GemListRow, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle};

pub fn sections(statuses: &HashMap<String, GemLatencyStatus>, stream: GemLatencyStatus) -> Vec<GemListSection> {
    let row = |title, host: &str, suffix: String| GemListRow::Latency {
        title,
        title_suffix: suffix,
        host: host.into(),
        status: statuses.get(host).cloned().unwrap_or(GemLatencyStatus::Loading),
    };
    vec![
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: vec![
                row(GemListRowTitle::Api, GEM_API_HOST, String::new()),
                GemListRow::Latency {
                    title: GemListRowTitle::Stream,
                    title_suffix: String::new(),
                    host: GEM_API_HOST.into(),
                    status: stream,
                },
            ],
        },
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: NodeRegion::all().into_iter().map(|region| row(GemListRowTitle::GemWalletNode, region.host(), format!(" {}", region.flag()))).collect(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_sections_group_api_and_stream_before_nodes() {
        let sections = sections(&HashMap::new(), GemLatencyStatus::Loading);
        assert_eq!(sections.len(), 2);
        assert!(sections.iter().all(|section| section.title == GemListSectionTitle::None));
        assert_eq!(
            sections[0].rows,
            vec![
                GemListRow::Latency {
                    title: GemListRowTitle::Api,
                    title_suffix: String::new(),
                    host: GEM_API_HOST.into(),
                    status: GemLatencyStatus::Loading
                },
                GemListRow::Latency {
                    title: GemListRowTitle::Stream,
                    title_suffix: String::new(),
                    host: GEM_API_HOST.into(),
                    status: GemLatencyStatus::Loading
                },
            ]
        );
        assert_eq!(sections[1].rows.len(), 3);
        for (row, region) in sections[1].rows.iter().zip(NodeRegion::all()) {
            assert_eq!(
                *row,
                GemListRow::Latency {
                    title: GemListRowTitle::GemWalletNode,
                    title_suffix: format!(" {}", region.flag()),
                    host: region.host().into(),
                    status: GemLatencyStatus::Loading
                }
            );
        }
    }

    #[test]
    fn test_statuses_keep_independent_results() {
        let fast = GemLatencyStatus::from(Some(Duration::from_millis(125)));
        let sections = sections(&HashMap::from([(GEM_API_HOST.into(), fast.clone()), (NodeRegion::Us.host().into(), GemLatencyStatus::Error)]), GemLatencyStatus::Error);
        assert!(matches!(&sections[0].rows[0], GemListRow::Latency { status, .. } if *status == fast));
        assert!(matches!(&sections[0].rows[1], GemListRow::Latency { status: GemLatencyStatus::Error, .. }));
        assert!(matches!(&sections[1].rows[0], GemListRow::Latency { status: GemLatencyStatus::Error, .. }));
    }
}
