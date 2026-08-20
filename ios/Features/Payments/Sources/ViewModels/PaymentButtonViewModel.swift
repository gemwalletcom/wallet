// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

enum PaymentButtonAction: Equatable {
    case collectData
    case confirm
    case tryAgain
}

struct PaymentButtonViewModel: StateButtonViewable {
    private let state: PaymentState
    private let perform: @MainActor @Sendable () -> Void

    init(
        state: PaymentState,
        onAction: @MainActor @Sendable @escaping () -> Void,
    ) {
        self.state = state
        perform = onAction
    }

    var buttonAction: PaymentButtonAction {
        if state.error != nil || state.isExpired {
            return .tryAgain
        }
        if state.needsDataCollection {
            return .collectData
        }
        return .confirm
    }

    var title: String {
        switch buttonAction {
        case .tryAgain: Localized.Common.tryAgain
        case .collectData, .confirm: Localized.Common.continue
        }
    }

    var icon: Image? {
        switch buttonAction {
        case .collectData: Images.System.person
        case .confirm, .tryAgain: .none
        }
    }

    var type: ButtonType {
        if state.isLoading {
            return .primary(.loading())
        }
        return .primary(.normal)
    }

    func action() {
        perform()
    }
}
