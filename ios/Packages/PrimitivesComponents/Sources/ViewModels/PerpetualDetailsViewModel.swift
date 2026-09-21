// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualConfirmDetails
import Localization
import Style

public struct PerpetualDetailsViewModel: Sendable, Identifiable {
    private let details: GemPerpetualConfirmDetails

    public init(details: GemPerpetualConfirmDetails) {
        self.details = details
    }

    public var id: String {
        details.id
    }

    public var title: String {
        Localized.Common.details
    }

    public var listItemModel: ListItemModel {
        ListItemModel(
            title: title,
            subtitle: details.summary.text?.text,
            subtitleStyle: TextStyle(font: .callout, color: details.summary.tone.color),
        )
    }
}

// MARK: - ListSectionProvideable

extension PerpetualDetailsViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemListSectionRow>] {
        details.sections.listSections
    }
}
