// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.swapperProviderConfig
import func Gemstone.swapperProviderFromStr
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
        let providerName = swapperProviderFromStr(s: providerId).map { swapperProviderConfig(provider: $0).name } ?? providerId

        return .listItem(
            .text(
                title: Localized.Common.provider,
                subtitle: providerName,
            ),
        )
    }
}
