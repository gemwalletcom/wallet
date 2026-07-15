// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Components
import Localization
import Primitives

struct ConfirmErrorViewModel {
    private let error: Error?
    private let onSelectListError: (Error) -> Void

    init(
        error: Error?,
        onSelectListError: @escaping (Error) -> Void,
    ) {
        self.error = error
        self.onSelectListError = onSelectListError
    }
}

// MARK: - ItemModelProvidable

extension ConfirmErrorViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        guard let error else { return .empty }
        return .error(
            title: Localized.Errors.errorOccurred,
            error: ChainCoreError.fromError(error) ?? error,
            onInfoAction: { onSelectListError(error) },
        )
    }
}
