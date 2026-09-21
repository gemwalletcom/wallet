// Copyright (c). Gem Wallet. All rights reserved.

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
