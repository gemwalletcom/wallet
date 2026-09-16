// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemCollectibleRow
import Localization
import Primitives

extension ReportReason {
    var title: String {
        switch self {
        case .spam: Localized.Nft.Report.Reason.spam
        case .malicious: Localized.Nft.Report.Reason.malicious
        case .inappropriate: Localized.Nft.Report.Reason.inappropriate
        case .copyright: Localized.Nft.Report.Reason.copyright
        case .other: Localized.Nft.Report.Reason.other
        }
    }
}

extension GemCollectibleRow {
    var title: String {
        switch self {
        case .collection: Localized.Nft.collection
        case .network: Localized.Transfer.network
        case .contract: Localized.Asset.contract
        case .tokenId: Localized.Asset.tokenId
        }
    }
}
