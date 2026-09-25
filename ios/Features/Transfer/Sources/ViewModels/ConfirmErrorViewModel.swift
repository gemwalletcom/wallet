// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmError
import Localization
import Primitives

struct ConfirmErrorViewModel {
    private let error: GemConfirmError?
    private let onSelectListError: (GemConfirmError) -> Void

    init(
        error: GemConfirmError?,
        onSelectListError: @escaping (GemConfirmError) -> Void,
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
            error: error,
            onInfoAction: error.display().hasInfoSheet() ? { onSelectListError(error) } : nil,
        )
    }
}
