// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives

struct ConfirmErrorViewModel {
    private let error: ConfirmTransferError?
    private let onSelectListError: (ConfirmTransferError) -> Void

    init(
        error: ConfirmTransferError?,
        onSelectListError: @escaping (ConfirmTransferError) -> Void,
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
            error: error.displayError,
            onInfoAction: { onSelectListError(error) },
        )
    }
}
