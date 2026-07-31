// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives

struct ReceiveNetworkSelectorViewModel: SelectableListAdoptable, SelectableListNavigationAdoptable {
    typealias Item = ReceiveNetworkItem

    let state: StateViewType<SelectableListType<ReceiveNetworkItem>>
    var selectedItems: Set<ReceiveNetworkItem>
    let selectionType: SelectionType

    var title: String {
        Localized.Settings.Networks.title
    }

    var doneTitle: String {
        Localized.Common.done
    }

    init(assetIds: [AssetId], selectedAssetId: AssetId) {
        let items = assetIds.map(ReceiveNetworkItem.init)
        self.init(
            state: .data(.plain(items)),
            selectedItems: items.filter { $0.assetId == selectedAssetId },
            selectionType: .checkmark,
        )
    }

    init(
        state: StateViewType<SelectableListType<ReceiveNetworkItem>>,
        selectedItems: [ReceiveNetworkItem],
        selectionType: SelectionType,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
    }
}
