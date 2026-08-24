// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization

struct FeeAssetsViewModel: SelectableListAdoptable {
    typealias Item = FeeAssetItem

    let state: StateViewType<SelectableListType<FeeAssetItem>>
    var selectedItems: Set<FeeAssetItem>
    let selectionType: SelectionType

    init(
        state: StateViewType<SelectableListType<FeeAssetItem>>,
        selectedItems: [FeeAssetItem],
        selectionType: SelectionType,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
    }
}

extension FeeAssetsViewModel: SelectableListNavigationAdoptable {
    var title: String { Localized.Assets.selectAsset }
    var doneTitle: String { Localized.Common.done }
}
