// Copyright (c). Gem Wallet. All rights reserved.

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

    public var title: String {
        switch status {
        case .online: ""
        case .noInternet: "No internet connection"
        case .noService: "No service connection"
        }
    }

    public var icon: Image {
        Images.System.exclamationmarkTriangle
    }

    public var iconColor: Color {
        Colors.orange
    }
}
