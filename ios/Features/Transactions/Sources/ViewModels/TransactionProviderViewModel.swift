// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.SwapProviderConfig
import Localization
import Primitives
import PrimitivesComponents

struct TransactionProviderViewModel {
    private let metadata: TransactionSwapMetadata?

    init(metadata: TransactionSwapMetadata?) {
        self.metadata = metadata
    }
}

extension TransactionProviderViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let providerId = metadata?.provider else {
            return .empty
        }

        return .listItem(
            .text(
                title: Localized.Common.provider,
                subtitle: SwapProviderConfig.fromString(id: providerId).inner().name,
            ),
        )
    }
}
