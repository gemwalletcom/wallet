// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import Style
import SwiftUI

public struct ConnectionStatusViewModel {
    private let status: ConnectionStatus

    public init(status: ConnectionStatus) {
        self.status = status
    }

    public var isVisible: Bool {
        status != .online
    }

    public var title: String? {
        switch status {
        case .online: .none
        case .noInternet: Localized.Errors.noInternetConnection
        case .noService: Localized.Errors.noServiceConnection
        }
    }

    public var subtitle: String {
        Localized.Errors.balancesActivityOutdated
    }

    public var icon: Image {
        Images.System.exclamationmarkTriangle
    }

    public var iconColor: Color {
        Colors.orange
    }
}
