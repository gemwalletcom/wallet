// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.Resource
import GemstonePrimitives
import Localization
import PrimitivesComponents

struct TransactionResourceViewModel {
    private let resource: Gemstone.Resource?

    init(resource: Gemstone.Resource?) {
        self.resource = resource
    }
}

extension TransactionResourceViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let resource else { return .empty }
        return .listItem(ListItemModel(title: Localized.Stake.resource, subtitle: ResourceViewModel(resource: resource.map()).title))
    }
}
