// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents

struct TransactionProviderViewModel {
    private let name: String?

    init(name: String?) {
        self.name = name
    }
}

extension TransactionProviderViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let name else {
            return .empty
        }
        return .listItem(
            .text(
                title: Localized.Common.provider,
                subtitle: name,
            ),
        )
    }
}
