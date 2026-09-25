// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives

struct ReceiveNetworkSelectorViewModel: SelectableListAdoptable, SelectableListNavigationAdoptable {
    typealias Item = AssetId

    let state: StateViewType<SelectableListType<AssetId>>
    var selectedItems: Set<AssetId>
    let selectionType: SelectionType

    var title: String {
        Localized.Settings.Networks.title
    }

    init(assetIds: [AssetId]) {
        self.init(state: .data(.plain(assetIds)))
    }

    init(
        state: StateViewType<SelectableListType<AssetId>>,
        selectedItems: [AssetId] = [],
        selectionType: SelectionType = .navigationLink,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
    }
}
