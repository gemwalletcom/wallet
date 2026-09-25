// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemWalletRow
import struct Gemstone.GemWalletSection
import Localization
import Primitives
import Style

public struct SelectWalletViewModel: SelectableListAdoptable {
    public typealias Item = GemWalletRow

    public var title: String {
        Localized.Wallets.title
    }

    public var state: StateViewType<SelectableListType<GemWalletRow>>
    public var selectedItems: Set<GemWalletRow>
    public var selectionType: SelectionType = .checkmark

    public init(
        sections: [GemWalletSection],
        selectedRow: GemWalletRow,
    ) {
        let sections = sections.map { section in
            switch section.kind {
            case .pinned: ListSection(id: "pinned", title: Localized.Common.pinned, image: Images.System.pin, values: section.rows)
            case .wallets: ListSection(id: "wallets", title: nil, image: nil, values: section.rows)
            }
        }

        self.init(
            state: .data(.section(sections)),
            selectedItems: [selectedRow],
            selectionType: .checkmark,
        )
    }

    public init(
        state: StateViewType<SelectableListType<GemWalletRow>>,
        selectedItems: [GemWalletRow],
        selectionType _: SelectionType,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
    }
}

extension SelectWalletViewModel: SelectableListNavigationAdoptable {}
