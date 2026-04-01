// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Style
import SwiftUI

public extension StateEmptyView where Content == EmptyView {
    static func noData() -> StateEmptyView<EmptyView> {
        StateEmptyView(title: Localized.Common.notAvailable)
    }

    static func error(_ error: Error) -> StateEmptyView<EmptyView> {
        StateEmptyView(
            title: Localized.Errors.errorOccured,
            description: error.localizedDescription,
            image: Images.ErrorConent.error,
        )
    }
}
